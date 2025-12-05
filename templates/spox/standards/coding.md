## Coding Standards

### Core Principles

1. **KISS** Keep it simple stupid. Simpler is always better. Reduce complexity as much as possible.
2. **Boy scout rule**. Leave the campground cleaner than you found it.
3. **Always find root cause**. Always look for the root cause of a problem.
4. **DRY**: Eliminate duplication. Extract shared logic instead of copy-pasting.
5. **YAGNI**: Build only what is required now. Add complexity only when evidence demands it.
6. **Single Responsibility**: One function, one purpose. If it needs “and” to describe it, split it.

### Design
1. Keep configurable data at high levels.
2. Prefer polymorphism over if/else or switch/case.
3. Isolate multi-threading code.
4. Avoid over-configurability.
5. Use dependency injection.
6. Follow Law of Demeter—classes know only direct dependencies.

### Understandability
1. Be consistent in style and approach.
2. Use explanatory variables.
3. Encapsulate boundary conditions in one place.
4. Prefer value objects over primitives.
5. Avoid logical dependencies within a class.
6. Avoid negative conditionals.

### Naming
1. Choose descriptive, unambiguous names.
2. Make meaningful distinctions.
3. Use pronounceable, searchable names.
4. Replace magic numbers with named constants.
5. Avoid prefixes and type encodings.

### Functions
1. Keep small.
2. Do one thing.
3. Use descriptive names.
4. Minimize arguments.
5. Avoid side effects.
6. No flag arguments—split into separate methods.

### Comments
1. Prefer self-documenting code.
2. Avoid redundancy and obvious noise.
3. No closing brace comments.
4. Delete commented-out code.
5. Use only for intent, clarification, or warnings.
6. Never use comments to track work.

### Structure
1. Separate concepts vertically.
2. Keep related code vertically dense.
3. Declare variables near usage.
4. Place dependent and similar functions close together.
5. Order functions top-down by call hierarchy.
6. Keep lines short; avoid horizontal alignment.
7. Use whitespace to show relationships.
8. Maintain consistent indentation.

### Objects & Data Structures
1. Hide internal structure.
2. Prefer pure data structures or objects—avoid hybrids.
3. Keep classes small, focused, with few instance variables.
4. Base classes shouldn't know their derivatives.
5. Prefer many small functions over flag-based behavior selection.
6. Prefer instance methods over static methods.

### Code Smells
1. **Rigidity**: Small changes cascade.
2. **Fragility**: Single changes break many places.
3. **Immobility**: Code not reusable elsewhere.
4. **Needless complexity**.
5. **Needless repetition**.
6. **Opacity**: Hard to understand.

### Code Organization

**Imports**: Standard → third-party → local. Remove unused imports.

**Dead Code**: Delete unused or commented-out code. Rely on version control for history.

**Function Size**: Keep functions small. Extract complex logic into helpers.