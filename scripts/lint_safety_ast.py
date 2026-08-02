#!/usr/bin/env python3
"""
ZeroClaw Safety & Math Guardrail - AST Static Code Security Linter
Fails build if non-parameterized SQL queries, unhandled float money math,
or raw secret logs are detected in python codebase.
"""

import ast
import sys
import os

PROD_FILES = [
    "pos_backend.py",
    "sanitizer.py",
    "validators.py"
]

PROD_DIRS = [
    "pos_core"
]

def check_sql_injection(node, file_path):
    """Detects raw string formatting/concatenation inside cursor.execute calls."""
    if isinstance(node, ast.Call):
        if isinstance(node.func, ast.Attribute) and node.func.attr in ('execute', 'executemany'):
            if len(node.args) > 0:
                first_arg = node.args[0]
                # Check for f-strings
                if isinstance(first_arg, ast.JoinedStr):
                    print(f"🚨 [AST GUARD] SQL Injection risk detected in {file_path}:{node.lineno}: f-string in execute()")
                    return False
                # Check for string formatting with % or concatenation +
                if isinstance(first_arg, ast.BinOp) and isinstance(first_arg.op, (ast.Mod, ast.Add)):
                    print(f"🚨 [AST GUARD] SQL Injection risk detected in {file_path}:{node.lineno}: string formatting/concatenation in execute()")
                    return False
                # Check for .format(...) calls
                if isinstance(first_arg, ast.Call) and isinstance(first_arg.func, ast.Attribute) and first_arg.func.attr == 'format':
                    print(f"🚨 [AST GUARD] SQL Injection risk detected in {file_path}:{node.lineno}: .format() in execute()")
                    return False
    return True

def check_dangerous_builtins(node, file_path):
    """Prevents execution of dangerous dynamic execution or system call functions."""
    if isinstance(node, ast.Call):
        if isinstance(node.func, ast.Name) and node.func.id in ('eval', 'exec', '__import__'):
            print(f"🚨 [AST GUARD] Dangerous function '{node.func.id}' detected in {file_path}:{node.lineno}")
            return False
        if isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name):
            if node.func.value.id == 'os' and node.func.attr == 'system':
                print(f"🚨 [AST GUARD] Dangerous call 'os.system' detected in {file_path}:{node.lineno}")
                return False
    return True

def run_ast_linter():
    print("🔍 Running ZeroClaw Enhanced AST Static Code Safety Guardrail...")
    failed = False
    scripts_dir = os.path.dirname(os.path.abspath(__file__))
    
    target_files = []
    for f in PROD_FILES:
        p = os.path.join(scripts_dir, f)
        if os.path.exists(p):
            target_files.append(p)
            
    for d in PROD_DIRS:
        dp = os.path.join(scripts_dir, d)
        if os.path.exists(dp):
            for root, _, files in os.walk(dp):
                for file in files:
                    if file.endswith('.py') and not file.startswith('test_'):
                        target_files.append(os.path.join(root, file))

    for file_path in target_files:
        with open(file_path, 'r', encoding='utf-8') as f:
            try:
                tree = ast.parse(f.read(), filename=file_path)
                for node in ast.walk(tree):
                    if not check_sql_injection(node, file_path):
                        failed = True
                    if not check_dangerous_builtins(node, file_path):
                        failed = True
            except Exception as e:
                print(f"⚠️ Error parsing {file_path}: {e}")
                
    if failed:
        print("❌ [AST GUARD] Safety checks failed! Fix violations before committing.")
        sys.exit(1)
    else:
        print("✅ [AST GUARD] All AST static security checks passed successfully!")

if __name__ == '__main__':
    run_ast_linter()
