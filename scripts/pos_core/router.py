#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Lightweight stdlib HTTP Micro-Router
Parses clean URL paths with urllib.parse to strip Query Parameters and prevent 404 errors.
"""

import json
from urllib.parse import urlparse, parse_qs
from sanitizer import redact_api_key

ROUTES_GET = {}
ROUTES_POST = {}

def route_get(path: str):
    def decorator(func):
        ROUTES_GET[path] = func
        return func
    return decorator

def route_post(path: str):
    def decorator(func):
        ROUTES_POST[path] = func
        return func
    return decorator

def send_json_response(handler, status_code: int, body, extra_headers: dict = None):
    """Sends standardized JSON HTTP responses with CORS and custom headers."""
    handler.send_response(status_code)
    handler.send_header('Content-Type', 'application/json')
    handler.send_header('Access-Control-Allow-Origin', '*')
    if extra_headers:
        for k, v in extra_headers.items():
            handler.send_header(k, v)
    handler.end_headers()
    handler.wfile.write(json.dumps(body, indent=2).encode('utf-8'))

def dispatch_request(handler, method: str, post_data: dict = None):
    """Dispatches HTTP GET and POST requests by matching clean path (without query params)."""
    routes = ROUTES_GET if method == 'GET' else ROUTES_POST
    parsed_url = urlparse(handler.path)
    raw_path = parsed_url.path or '/'
    clean_path = raw_path.rstrip('/') if raw_path != '/' else '/'
    query_params = parse_qs(parsed_url.query)

    if clean_path in routes:
        route_func = routes[clean_path]
        try:
            if method == 'POST':
                res = route_func(handler, post_data, query_params)
            else:
                res = route_func(handler, query_params)
        except Exception as e:
            send_json_response(handler, 500, {"error": redact_api_key(str(e))})
            return

        if isinstance(res, tuple):
            if len(res) == 2:
                status_code, body = res
                send_json_response(handler, status_code, body)
            elif len(res) == 3:
                status_code, body, extra_headers = res
                send_json_response(handler, status_code, body, extra_headers)
    else:
        send_json_response(handler, 404, {"error": "Endpoint not found"})
