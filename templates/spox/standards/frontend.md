## UI Standards

### UI accessibility best practices

**Required:**
- Semantic HTML (`<nav>`, `<main>`, `<button>`, not `<div>` for everything)
- Keyboard navigable with visible focus states
- Alt text on images, labels on form inputs
- Heading hierarchy: h1 → h2 → h3 (no skipping)

**Color:** 4.5:1 contrast minimum; never color-only information

**ARIA:** Only when semantic HTML insufficient

**Dynamic content:** Manage focus on modals, route changes, live updates

**Verify:** Test with screen reader before shipping

### UI component best practices

**Design principles:**
- Single responsibility (one purpose per component)
- Composable (combine small components, not monoliths)
- Reusable (configurable via props)
- Encapsulated (hide internals, expose minimal API)

**Props:**
- Explicit types with sensible defaults
- Keep count low; many props → split component
- Document with examples

**State:** Keep local; lift only when shared

**Naming:** Descriptive, consistent with project conventions

### CSS best practices

**Rules:**
- Follow project methodology (Tailwind/BEM/modules) consistently
- Use design tokens (colors, spacing, typography)
- Work with framework patterns, don't override
- Minimize custom CSS

**Production:** Enable purging/tree-shaking for unused styles

### Responsive design best practices

**Approach:** Mobile-first, progressive enhancement to large screens

**Layout:**
- Fluid containers (%, fr)
- Relative units (rem/em over px)
- Standard breakpoints (mobile → tablet → desktop)

**Touch:** Minimum 44×44px tap targets

**Typography:** Readable at all breakpoints without zoom

**Performance:** Optimize images/assets for mobile

**Verify:** Test on real devices across breakpoints