'use strict';


let mod = null;
const mod_promise = import('./pkg/wasm.js').then(
    m => (mod = m, mod.default()));

class PolysubmlDemo extends HTMLElement {
  constructor() {
    // Always call super first in constructor
    super();

    // Create a shadow root
    // const shadow = this.attachShadow({mode: 'open'});
    // shadow.innerHTML = HTML;
    const shadow = this.attachInternals().shadowRoot;

    mod_promise.then(
        wasm => initializeRepl(shadow, mod.State.new(), Printer)).catch(
        e => {shadow.getElementById('loading').textContent = 'Failed to load demo: ' + e});

  }
}
customElements.define('polysubml-demo', PolysubmlDemo);


function initializeRepl(root, compiler, Printer) {
    console.log('Initializing REPL');
    const container = root.getElementById('container');
    const output = root.getElementById('output');
    const prompt = root.getElementById('prompt');
    const editor = root.getElementById('editor');

    function addOutput(line, cls) {
        const l = document.createElement('pre');
        l.textContent = line;
        if (cls) {
            l.classList.add(cls);
        }
        output.appendChild(l);
        return l;
    }

    const $ = Object.create(null);
    const history = [];
    let history_offset = -1;

    function execCode(script) {
        let compiled;
        try {
            if (!compiler.process(script)) {return [false, compiler.get_err()];}
            compiled = '(' + compiler.get_output() + ')';
        } catch (e) {
            return [false, 'Internal compiler error: ' + e.toString() +
                '\nIf you see this message, please file an issue on Github with the code required to trigger this error.'];
        }

        try {
            const p = new Printer;
            const val = eval(compiled);
            p.visitRoot(val);
            return [true, p.parts.join('')];
        } catch (e) {
            return [false, 'An error occurred during evaluation in the repl: ' + e.toString()];
        }
    }

    function processCode(script) {
        const [success, res] = execCode(script);
        addOutput(res, success ? 'success' : 'error');
        // scroll output window to the bottom
        output.scrollTop = output.scrollHeight;
        return success;
    }


    function processReplInput(line) {
        line = line.trim();
        if (!line) {return;}

        history_offset = -1;
        if (history[history.length-1] !== line) {history.push(line);}
        // \u00a0 = non breaking space
        addOutput('>>\u00a0' + line, 'input');
        processCode(line);
    }

    root.getElementById('compile-and-run').addEventListener('click', e => {
        const s = editor.value.trim();
        if (!s) {return;}

        // Clear repl output
        output.textContent = '';
        compiler.reset();
        if (processCode(s)) {prompt.focus({preventScroll: true})}
    });

    // Implement repl command history
    prompt.addEventListener('keydown', e => {
        switch (e.key) {
            case 'ArrowDown': history_offset -= 1; break;
            case 'ArrowUp': history_offset += 1; break;
            default: return;
        }
        e.preventDefault();

        if (history_offset >= history.length) {history_offset = history.length - 1;}
        if (history_offset < 0) {history_offset = 0;}
        prompt.value = history[history.length - history_offset - 1];
    });

    // If they click in the space below the prompt, focus on the prompt to make it easier to select
    root.getElementById('space-below-prompt').addEventListener('click', e => {
        e.preventDefault();
        prompt.focus({preventScroll: true});
    });

    root.getElementById('rhs-form').addEventListener('submit', e => {
        e.preventDefault();
        const s = prompt.value.trim();
        prompt.value = '';

        if (!s) {return;}
        processReplInput(s);
    });

    container.classList.remove('loading');
    prompt.disabled = false;
    container.removeChild(root.getElementById('loading'));
    console.log('Initialized REPL');

    // Run the example code
    processCode(editor.value.trim())
}

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

    // print(e) {this.visit(e); return this.parts.join('');}
}

// This function exists to be called from within PolySubML code
// and is available implicitly when we eval() the compiled code in execCode.
function loop(expr) {
    let v = expr();
    while (v.$tag === 'Continue') {
        v = expr();
    }
    return v.$val;
}