// Minimal FileReader so GLTFExporter (binary mode) runs under Node.
globalThis.FileReader ??= class { readAsArrayBuffer(b) { b.arrayBuffer().then((r) => { this.result = r; this.onloadend?.(); }); }
  readAsDataURL(b) { b.arrayBuffer().then((r) => { this.result = `data:${b.type};base64,` + Buffer.from(r).toString('base64'); this.onloadend?.(); }); } };
