## Backend

### API endpoint standards and conventions

`{resources}` is exemplary for the endpoints to implement:

```
GET    /{resources}      → list
GET    /{resources}/:id  → show
POST   /{resources}      → create
PUT    /{resources}/:id  → replace
PATCH  /{resources}/:id  → update
DELETE /{resources}/:id  → destroy
```

**URL Rules:**
- Plural nouns: `/users`, `/products`
- Lowercase, hyphenated: `/user-profiles`
- Max 2-3 nesting levels: `/users/:id/orders`
- Query params for filtering/sorting/pagination

**Responses:**
- 200 OK, 201 Created, 204 No Content
- 400 Bad Request, 401 Unauthorized, 403 Forbidden, 404 Not Found
- 500 Internal Server Error

**Headers:** Include rate limit info (`X-RateLimit-Limit`, `X-RateLimit-Remaining`)

**Versioning:** Use `/v1/` prefix or `Accept` header

### Database migration best practices

**Every migration must:**
- Have a working rollback/down method
- Make one logical change only
- Use descriptive name: `add_email_index_to_users`

**For production:**
- Schema changes separate from data migrations
- Concurrent index creation on large tables
- Test rollback before deploying
- Consider backwards compatibility for zero-downtime

### Database model best practices

**Required on all tables:**
- `created_at`, `updated_at` timestamps
- Primary key (prefer UUID or auto-increment)

**Constraints:**
- NOT NULL where data is required
- UNIQUE for natural keys
- Foreign keys with appropriate CASCADE

**Indexes:**
- All foreign key columns
- Frequently filtered/sorted columns
- Composite indexes for common query patterns

**Naming:**
- Models: singular (`User`)
- Tables: plural (`users`)
- Foreign keys: `{table}_id`

### Database query best practices

**Security (example representative for all languages):**
```python
# ✓ Parameterized
db.query("SELECT * FROM users WHERE id = ?", [user_id])

# ✗ NEVER interpolate
db.query(f"SELECT * FROM users WHERE id = {user_id}")
```

**Performance:**
- Select specific columns, not `SELECT *`
- Only select what is required
- Eager load relations to prevent N+1
- Prefer joins for eager loading, avoid subqueries
- Use transactions for related writes
- Set query timeouts
- Cache expensive queries

**Indexes:** Add to columns in WHERE, JOIN, ORDER BY clauses