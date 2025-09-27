// PolySubML JavaScript Runtime
class Printer {
    constructor() {
        this.parts = [];
        this.seen = new WeakSet;
        this.current_size = 0;
    }

    push(s) {
        this.parts.push(s);
        this.current_size += s.length;
    }

    visitRoot(e) {
        this.seen = new WeakSet;
        this.current_size = 0;
        this.visit(e);
    }

    visit(e) {
        const type = typeof e;
        if (type === 'boolean' || type === 'bigint') {this.push(e.toString()); return;}
        if (type === 'string') {this.push(JSON.stringify(e)); return;}
        if (type === 'number') {
            let s = e.toString();
            if (/^-?\d+$/.test(s)) {s += '.0'}
            this.push(s);
            return;
        }
        if (type === 'function') {this.push('<fun>'); return;}
        if (type === 'symbol') {this.push('<sym>'); return;}
        if (e === null) {this.push('null'); return;}
        if (e === undefined) {this.push('<undefined>'); return;}

        if (this.seen.has(e)) {this.push('...'); return;}
        this.seen.add(e);

        const LIMIT = 80;
        if (this.current_size > LIMIT) {this.push('...'); return;}

        if (e.$tag) {
            this.push(e.$tag);
            if (!e.$val || typeof e.$val !== 'object') {
                this.push(' ');
            }
            this.visit(e.$val);
        } else {
            // Tuple-like objects
            const entries = new Map(Object.entries(e));
            if (entries.size >= 2 && [...Array(entries.size).keys()].every(i => entries.has('_'+i))) {
                this.push('(');
                for (let i=0; i < entries.size; ++i) {
                    if (i>0) {this.push(', ')}
                    if (this.current_size > LIMIT) {this.push('...'); break;}

                    this.visit(entries.get('_'+i));
                }
                this.push(')');
            } else {
                this.push('{');
                let first = true;
                for (const [k, v] of entries) {
                    if (!first) {this.push('; ')}
                    first = false;
                    if (this.current_size > LIMIT) {this.push('...'); break;}

                    this.push(k + '=');
                    this.visit(v);
                }
                this.push('}');
            }
        }
    }

    println(...args) {
        for (let arg of args) {
            if (typeof arg === 'string') {
                this.push(arg);
            } else {
                this.visitRoot(arg);
            }
            this.push(' ');
        }
        this.parts.pop();
        this.push('\n');
    }
}

// Helper function for PolySubML loop constructs
function loop(expr) {
    let v = expr();
    while (v.$tag === 'Continue') {
        v = expr();
    }
    return v.$val;
}

// Global print function
const printer = new Printer();
function print(...args) {
    printer.println(...args);
    process.stdout.write(printer.parts.join(''));
    printer.parts = [];
}

// Execute PolySubML compiled code (matching web demo approach)
function execute(compiledCode) {
    try {
        const $ = Object.create(null);
        const p = new Printer();

        // Execute the compiled code with p in scope
        const result = eval(compiledCode);

        // If there's output from print statements, show it
        if (p.parts.length > 0) {
            process.stdout.write(p.parts.join(''));
        }

        // If there's a return value and it's not undefined, print it
        if (result !== undefined) {
            const outputPrinter = new Printer();
            outputPrinter.visitRoot(result);
            const output = outputPrinter.parts.join('');
            if (output.trim()) {
                console.log(output);
            }
        }
    } catch (e) {
        console.error('Runtime error:', e.message);
        process.exit(1);
    }
}