import os
import re

TARGETS = ["Expr::Variable", "Pattern::Variable", "PolicyExpr::Var"]

def span_path_for_file(path):
    if "ash-core" in path:
        if "/src/" in path or "/tests/" in path:
            return "crate::ast::Span::default()"
        return "ash_core::ast::Span::default()"
    elif "ash-parser" in path:
        if "/src/" in path:
            return "crate::token::Span::default()"
        return "ash_parser::token::Span::default()"
    elif "ash-typeck" in path:
        return "ash_parser::token::Span::default()"
    elif "ash-interp" in path or "ash-engine" in path:
        return "ash_core::ast::Span::default()"
    elif "ash-fuzz" in path:
        return "ash_core::ast::Span::default()"
    elif "ash-repl" in path:
        # repl only patterns, no constructors expected
        return "Span::default()"
    else:
        return "Span::default()"

def find_closing_paren(text, start):
    depth = 1
    i = start + 1
    while i < len(text) and depth > 0:
        if text[i] == '(':
            depth += 1
        elif text[i] == ')':
            depth -= 1
        i += 1
    return i

def is_inside_comment_or_string(line, pos):
    # inside string literal
    if line[:pos].count('"') % 2 == 1:
        return True
    # inside line comment
    comment_start = line.find('//')
    if comment_start != -1 and comment_start < pos:
        return True
    return False

def transform_line(line, span_path):
    for target in TARGETS:
        idx = 0
        while True:
            pos = line.find(target, idx)
            if pos == -1:
                break
            paren_pos = line.find('(', pos)
            if paren_pos == -1:
                idx = pos + len(target)
                continue
            if is_inside_comment_or_string(line, pos):
                idx = pos + len(target)
                continue
            end_pos = find_closing_paren(line, paren_pos)
            inner = line[paren_pos+1:end_pos-1]
            before = line[:pos]
            after = line[end_pos:]

            # Heuristic: simple identifier/ref patterns
            is_simple = re.fullmatch(r'\s*(_|ref\s+mut\s+\w+|ref\s+\w+|mut\s+\w+|\w+)\s*', inner) is not None

            # Determine if we are in a pattern context
            in_pattern = False
            rest_of_line = line[end_pos:]
            if '=>' in rest_of_line:
                in_pattern = True
            elif 'matches!' in line:
                in_pattern = True
            elif line.strip().startswith('if let ') or line.strip().startswith('while let '):
                in_pattern = True
            elif 'let ' in before and '=' in rest_of_line:
                in_pattern = True

            if is_simple and in_pattern:
                inner_stripped = inner.strip()
                if inner_stripped == '_':
                    replacement = f'{target} {{ .. }}'
                elif inner_stripped.startswith('ref '):
                    rest = inner_stripped[4:].strip()
                    if rest.startswith('mut '):
                        name = rest[4:].strip()
                        replacement = f'{target} {{ name: ref mut {name}, .. }}'
                    else:
                        replacement = f'{target} {{ name: ref {rest}, .. }}'
                else:
                    replacement = f'{target} {{ name: {inner_stripped}, .. }}'
            else:
                # Constructor
                if not inner.strip():
                    replacement = f'{target} {{ name: Default::default(), span: {span_path} }}'
                else:
                    replacement = f'{target} {{ name: {inner}, span: {span_path} }}'

            line = line[:pos] + replacement + line[end_pos:]
            idx = pos + len(replacement)
    return line

def fix_prop_map(content, span_path):
    # Fix .prop_map(Expr::Variable) -> .prop_map(|name| Expr::Variable { name, span: ... })
    for target in TARGETS:
        short = target.split('::')[-1]
        pattern = re.compile(r'(\.prop_map\()(' + re.escape(target) + r')(\))')
        def repl(m):
            return f'{m.group(1)}|name| {target} {{ name, span: {span_path} }}{m.group(3)}'
        content = pattern.sub(repl, content)
    return content

def process_file(path):
    with open(path, 'r') as f:
        content = f.read()
    span_path = span_path_for_file(path)
    new_lines = [transform_line(line, span_path) for line in content.splitlines()]
    new_content = '\n'.join(new_lines)
    new_content = fix_prop_map(new_content, span_path)
    if new_content != content:
        with open(path, 'w') as f:
            f.write(new_content)

for root, dirs, files in os.walk('/home/dikini/Projects/ash/.worktrees/phase-84/crates'):
    # skip target directories
    dirs[:] = [d for d in dirs if d != 'target']
    for f in files:
        if f.endswith('.rs'):
            process_file(os.path.join(root, f))
