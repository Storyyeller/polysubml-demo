use std::collections::HashMap;
use std::collections::HashSet;
use std::error;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Span(usize);

pub type Spanned<T> = (T, Span);

#[derive(Debug)]
struct Source {
    s: String,
    line_offsets: Box<[usize]>,
}
impl Source {
    fn new(mut s: String) -> Self {
        // Ensure trailing space so lines will display nicely
        if !s.ends_with('\n') {
            s.push('\n');
        }

        // Precalculate the offsets of each line in the string
        let line_offsets = s
            .lines()
            .map(|line| line.as_ptr() as usize - s.as_ptr() as usize)
            .chain(std::iter::once(s.len()))
            .collect();
        Self { s, line_offsets }
    }

    fn get_lineno(&self, off: usize) -> usize {
        match self.line_offsets.binary_search(&off) {
            Ok(ind) => ind,
            Err(ind) => ind - 1,
        }
    }

    fn get_pos(&self, off: usize) -> (usize, usize) {
        let lineno = self.get_lineno(off);
        let start = self.line_offsets[lineno];
        (lineno, off - start)
    }

    fn get_line(&self, lineno: usize) -> &str {
        if lineno + 1 >= self.line_offsets.len() {
            return "\n";
        }

        let off = self.line_offsets[lineno];
        let off2 = self.line_offsets[lineno + 1];
        &self.s[off..off2]
    }

    fn print_line_if_nonempty(&self, out: &mut String, lineno: usize) {
        let line = self.get_line(lineno);
        if !line.trim().is_empty() {
            *out += line;
        }
    }

    fn print_nonempty_lines(&self, out: &mut String, y1: usize, y2: usize) {
        for y in y1..y2 {
            self.print_line_if_nonempty(out, y);
        }
    }

    fn print_middle_context(&self, out: &mut String, y1: usize, y2: usize) {
        // Make range inclusive of start
        let y1 = y1 + 1;
        if y2 >= y1 + 2 + CONTEXT_LINES * 2 {
            let skipped = y2 - y1 - CONTEXT_LINES * 2;
            self.print_nonempty_lines(out, y1, y1 + CONTEXT_LINES);
            *out += &format!("... {} lines omitted\n", skipped);
            self.print_nonempty_lines(out, y2 - CONTEXT_LINES, y2);
        } else {
            self.print_nonempty_lines(out, y1, y2);
        }
    }
}

const CONTEXT_LINES: usize = 2;

#[derive(Debug, Default)]
pub struct SpanManager {
    sources: Vec<Source>,
    spans: Vec<(usize, usize, usize)>,
}
impl SpanManager {
    pub fn add_source(&mut self, source: String) -> SpanMaker {
        let i = self.sources.len();
        self.sources.push(Source::new(source));
        SpanMaker {
            parent: self,
            source_ind: i,
            pool: Default::default(),
        }
    }

    fn new_span(&mut self, source_ind: usize, l: usize, r: usize) -> Span {
        let i = self.spans.len();
        self.spans.push((source_ind, l, r));
        Span(i)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////
    /// Printing functions
    fn highlight_line(&self, out: &mut String, line: &str, parts: Vec<(&str, usize, usize)>) {
        *out += line;
        let mut pos = 0;

        for (s, mut start, end) in parts {
            start = std::cmp::max(start, pos);
            if start >= end {
                continue;
            }

            *out += &" ".repeat(start - pos);
            *out += &s.repeat(end - start);
            pos = end;
        }

        *out += &" ".repeat(line.len().saturating_sub(pos));
        *out += "\n";
    }

    fn print(&self, out: &mut String, span: Span) {
        let (source_ind, l, r) = self.spans[span.0];
        let source = &self.sources[source_ind];

        // println!("source offsets {} {} {:?}", source.s.len(), source.line_offsets.len(), source.line_offsets);
        let (y1, x1) = source.get_pos(l);
        let (y2, x2) = source.get_pos(r);

        // Extra leading line of context
        source.print_nonempty_lines(out, y1.saturating_sub(CONTEXT_LINES), y1);

        let line = source.get_line(y1);
        let start = x1;
        let end = if y1 == y2 { x2 } else { line.len() };
        self.highlight_line(out, line, vec![("^", start, start + 1), ("~", start + 1, end)]);

        source.print_middle_context(out, y1, y2);
        if y2 > y1 {
            let line = source.get_line(y2);
            self.highlight_line(out, line, vec![("~", 0, x2)]);
        }

        // Extra trailing line of context
        source.print_nonempty_lines(out, y2 + 1, y2 + 1 + CONTEXT_LINES);
    }

    fn print_insertion(&self, out: &mut String, before: &str, span: Span, after: &str) {
        let (source_ind, l, r) = self.spans[span.0];
        let source = &self.sources[source_ind];

        let insertions = [(before, source.get_pos(l)), (after, source.get_pos(r))];
        // Skip when insertion string is empty
        let insertions = insertions.iter().copied().filter(|t| !t.0.is_empty());
        // Group by line
        use itertools::Itertools;
        let insertions = insertions.chunk_by(|&(s, (y, x))| y);

        let mut prev = None;
        for (y, chunk) in insertions.into_iter() {
            if let Some(prev) = prev {
                source.print_middle_context(out, prev, y);
            } else {
                source.print_nonempty_lines(out, y.saturating_sub(CONTEXT_LINES), y);
            }
            prev = Some(y);

            let mut line = source.get_line(y).to_string();
            let mut inserted = 0;
            let mut highlights = Vec::new();

            for (s, (_, mut x)) in chunk {
                x += inserted;
                line.insert_str(x, s);
                inserted += s.len();
                highlights.push(("+", x, x + s.len()));
            }

            self.highlight_line(out, &line, highlights);
        }

        if let Some(y) = prev {
            source.print_nonempty_lines(out, y + 1, y + 1 + CONTEXT_LINES);
        }
    }
}

#[derive(Debug)]
pub struct SpanMaker<'a> {
    parent: &'a mut SpanManager,
    source_ind: usize,
    pool: HashMap<(usize, usize), Span>,
}
impl<'a> SpanMaker<'a> {
    pub fn span(&mut self, l: usize, r: usize) -> Span {
        // Make the borrow checker happy
        let source_ind = self.source_ind;
        let parent = &mut self.parent;

        *self.pool.entry((l, r)).or_insert_with(|| parent.new_span(source_ind, l, r))
    }
}

#[derive(Debug)]
enum Item {
    Str(String),
    Span(Span),
    Insert(String, Span, String),
}

#[derive(Debug)]
pub struct SpannedError {
    // pairs: Vec<(String, Span)>,
    items: Vec<Item>,
}

impl SpannedError {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push_str(&mut self, s: impl Into<String>) {
        self.items.push(Item::Str(s.into()));
    }

    pub fn push_span(&mut self, span: Span) {
        self.items.push(Item::Span(span));
    }

    pub fn push_insert(&mut self, before: impl Into<String>, span: Span, after: impl Into<String>) {
        self.items.push(Item::Insert(before.into(), span, after.into()));
    }

    pub fn new1(s1: impl Into<String>, s2: Span) -> Self {
        let mut new = Self::new();
        new.push_str(s1);
        new.push_span(s2);
        new
    }

    pub fn new2(s1: impl Into<String>, s2: Span, s3: impl Into<String>, s4: Span) -> Self {
        let mut new = Self::new();
        new.push_str(s1);
        new.push_span(s2);
        new.push_str(s3);
        new.push_span(s4);
        new
    }

    pub fn print(&self, sm: &SpanManager) -> String {
        let mut out = String::new();
        for item in self.items.iter() {
            match item {
                &Item::Str(ref s) => {
                    out += s;
                    out += "\n";
                }
                &Item::Span(span) => sm.print(&mut out, span),
                &Item::Insert(ref before, span, ref after) => sm.print_insertion(&mut out, before, span, after),
            }
        }
        out
    }
}
impl fmt::Display for SpannedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
impl error::Error for SpannedError {}
