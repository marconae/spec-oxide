## MCP Tools

Prefer MCP tools over text-based alternatives. MCP tools provide semantic understanding; text tools only match patterns.

### Serena MCP

**Use for:** Code understanding, code navigation, symbol lookup, reference finding, code analysis.

**Prefer over:** `rg`, `grep`, `find`, `ag`, `ast-grep`, or any text-based search.

| Task                      | Use Serena                | Not                    |
|---------------------------|---------------------------|------------------------|
| Find function definition  | `serena.find_definition`  | `rg "function foo"`    |
| Find all references       | `serena.find_references`  | `rg "foo("`            |
| Understand code structure | `serena.get_symbols`      | `rg "class\|function"` |
| Navigate to symbol        | `serena.go_to_symbol`     | `grep -rn "symbol"`    |
| Analyze dependencies      | `serena.get_dependencies` | Manual file reading    |

**Why:** Serena understands code semantically. It knows that `foo()` in file A calls the `foo` defined in file B. Text
search only finds string matches.

### Context7 MCP

**Use for:** Library documentation, API references, framework guides, package usage.

**Prefer over:** Training data, assumptions, outdated memory, web search for docs.

| Task                          | Use Context7        | Not                        |
|-------------------------------|---------------------|----------------------------|
| Check API signature           | `context7.lookup`   | Assume from training       |
| Find library examples         | `context7.examples` | Guess syntax               |
| Verify current behavior       | `context7.docs`     | Rely on outdated knowledge |
| Understand framework patterns | `context7.guides`   | Web search                 |

**Why:** Training data has a cutoff date. Libraries change. Context7 provides current, accurate documentation. When in
doubt, verify with Context7.

## Decision Flow

```
Need to understand code?
├─ Symbol, reference, or structure → Serena
├─ Library or API docs → Context7
└─ Neither available → Fall back to text tools
```

### Rules

1. **Serena first** for any code navigation or understanding task
2. **Context7 first** for any library or documentation lookup
3. **Text tools** only when MCP tools are unavailable or insufficient
4. **Never assume** library behavior—verify with Context7
5. **Never pattern-match** code—understand it with Serena