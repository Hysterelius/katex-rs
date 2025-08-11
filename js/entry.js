var global;
try {
    global = Function('return this')();
} catch (e) {
    global = window;
}
global.katexRenderToString = katex.renderToString;

if (global.temml) {
    global.temmlRenderToString = temml.renderToString;
}
