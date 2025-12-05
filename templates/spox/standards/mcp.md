## MCP Tools

### Rules

**Priority:** Serena (code) → Context7 (docs) → text tools (fallback only)

1. **Serena first** for code navigation, understanding, editing
2. **Context7 first** for library/API documentation
3. **Symbolic editing** over read/edit/write cycles
4. **Check onboarding** before first Serena use on any project
5. **Text tools** only when MCP tools unavailable
6. **Never assume** library behavior—verify with Context7
7. **Never pattern-match** code—use Serena's semantic understanding

### Serena MCP

Semantic code understanding and editing. **Always prefer over** `rg`, `grep`, `find`, `ag`, `ast-grep`, or
read/edit/write cycles.

#### Tool Reference

| Task                   | Use                        | Not                    |
|------------------------|----------------------------|------------------------|
| List directory         | `list_dir`                 | `ls`, `find`           |
| Find files             | `find_file`                | `find`, `rg --files`   |
| File symbols           | `get_symbols_overview`     | `rg "class\|function"` |
| Symbol definition      | `find_symbol`              | `rg "function foo"`    |
| Symbol references      | `find_referencing_symbols` | `rg "foo("`            |
| Update function body   | `replace_symbol_body`      | read → edit → write    |
| Add code after symbol  | `insert_after_symbol`      | read → edit → write    |
| Add code before symbol | `insert_before_symbol`     | read → edit → write    |
| Rename across codebase | `rename_symbol`            | `rg` + manual edits    |

#### Reflection Tools

- `think_about_collected_information` — after exploration
- `think_about_task_adherence` — during implementation
- `think_about_whether_you_are_done` — before completion

#### Workflow

```
Explore → find_symbol, get_symbols_overview
Understand → find_referencing_symbols
Reflect → think_about_collected_information
Edit → replace_symbol_body, insert_*_symbol
Verify → find_referencing_symbols
Check → think_about_whether_you_are_done
```

### Context7 MCP

Current library docs and API references. **Prefer over** training data, assumptions, or web search for documentation.

Use `context7` for: API signatures, library examples, framework patterns, current behavior verification.