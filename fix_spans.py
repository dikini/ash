import re
from pathlib import Path

core_span = 'ash_core::ast::Span::default()'
surface_span = 'ash_parser::token::Span::default()'

def find_matching_paren(text, start):
    """Find index of matching ) for ( at start."""
    depth = 1
    i = start + 1
    while i < len(text) and depth > 0:
        if text[i] == '(':
            depth += 1
        elif text[i] == ')':
            depth -= 1
        i += 1
    return i - 1

def replace_constructors(text, prefix, span_default):
    """Replace Prefix(inner) with Prefix { name: inner, span: span_default }."""
    pattern = prefix + '('
    idx = 0
    while True:
        idx = text.find(pattern, idx)
        if idx == -1:
            break
        open_paren = idx + len(pattern) - 1
        close_paren = find_matching_paren(text, open_paren)
        inner = text[open_paren + 1:close_paren]
        # Check if already struct variant (look backwards for ' {')
        before = text[idx - 1:idx]
        if before == '{':
            idx = close_paren + 1
            continue
        replacement = f'{prefix} {{ name: {inner}, span: {span_default} }}'
        text = text[:idx] + replacement + text[close_paren + 1:]
        idx += len(replacement)
    return text

def fix_file(path: Path):
    text = path.read_text()
    original = text
    is_surface = 'crates/ash-parser' in str(path)
    span_default = surface_span if is_surface else core_span

    text = replace_constructors(text, 'Expr::Variable', span_default)
    text = replace_constructors(text, 'Pattern::Variable', span_default)
    text = replace_constructors(text, 'PolicyExpr::Var', span_default)

    if text != original:
        path.write_text(text)
        return True
    return False

changed = 0
for p in Path('crates').rglob('*.rs'):
    if fix_file(p):
        changed += 1

print(f'Changed {changed} files')
