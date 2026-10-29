"""Static no-cache server + POST /frame?clip=&i= writes frames to /tmp/kk-frames/CLIP/NNNN.png (done=1 touches DONE)."""
import http.server, os, sys, urllib.parse
ROOT = '/tmp/kk-frames'
class H(http.server.SimpleHTTPRequestHandler):
    def end_headers(self): self.send_header('Cache-Control', 'no-store'); super().end_headers()
    def log_message(self, *a): pass
    def do_POST(self):
        q = dict(urllib.parse.parse_qsl(urllib.parse.urlparse(self.path).query)); d = os.path.join(ROOT, q['clip']); os.makedirs(d, exist_ok=True)
        data = self.rfile.read(int(self.headers.get('Content-Length', 0)))
        open(os.path.join(d, 'DONE' if 'done' in q else '%04d.png' % int(q['i'])), 'wb').write(data)
        self.send_response(204); self.end_headers()
http.server.ThreadingHTTPServer(('127.0.0.1', 18765), H).serve_forever()
