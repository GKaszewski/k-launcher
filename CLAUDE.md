## 1. Core Philosophy

- **Test-Driven Development (TDD):** No functional code is written without a failing test first. Red-Green-Refactor is the mandatory cycle.
- **Clean Architecture:** Maintain strict separation between Domain, Application, and Infrastructure layers.
- **Newtype Pattern:** Use the "Newtype" pattern for all domain primitives (e.g., `struct UserId(Uuid)`) to ensure type safety and prevent primitive obsession.

---

## 2. Structural Rules

### Dependency Management

- **Strict Unidirectionality:** Dependencies must only point inwards (towards the Domain).
- **No Cyclic Dependencies:** Use traits and Dependency Injection (DI) to break cycles. If two modules need each other, abstract the shared behavior into a trait or move common data to a lower-level module.
- **Feature Gating:** Organize the project into logical crates or modules that can be compiled independently.

### Traits and Decoupling

- **Swappable Infrastructure:** Define all external interactions (Database, API, File System) as traits in the Application layer.
- **Small Traits:** Adhere to the Interface Segregation Principle. Favor many specific traits over one "God" trait.
- **Mocking:** Use traits to allow easy mocking in unit tests without requiring a real database or network.

---

## 3. Rust Specifics & Clean Code

### Type Safety

- Avoid `String` or `i32` for domain concepts. Wrap them in structs.
- Use `Result` and `Option` explicitly. Minimize `unwrap()` and `expect()`—handle errors gracefully at the boundaries.

### Formatting & Style

- Follow standard `rustfmt` and `clippy` suggestions.
- Function names should be descriptive (e.g., `process_valid_order` instead of `handle_data`).
- Keep functions small (typically under 20-30 lines).

---

## 4. Layer Definitions

| Layer              | Crates                                          | Responsibility                                | Allowed Dependencies |
| ------------------ | ----------------------------------------------- | --------------------------------------------- | -------------------- |
| **Domain**         | `k-launcher-domain`                             | Pure value types, newtypes, constants, port traits (Plugin, AppLauncher). | None (Pure Rust + serde + async-trait) |
| **Application**    | `k-launcher-kernel`                              | Kernel orchestrator. | Domain |
| **Infrastructure** | `k-launcher-ui`, `k-launcher-ui-egui`, `k-launcher-ui-core`, `k-launcher-os-bridge`, `k-launcher-plugin-host`, `k-launcher-config`, all `plugin-*` crates | Trait implementations, UI, config, plugins. | Domain, Application |
| **Main**           | `k-launcher`                                    | Entry point, DI wiring, logging.              | All of the above |

---

## 5. TDD Workflow Requirement

1. **Write a Test:** Create a test in `src/lib.rs` or a `tests/` directory.
2. **Define the Interface:** Use a trait or function signature to make the test compile (but fail).
3. **Minimum Implementation:** Write just enough code to pass the test.
4. **Refactor:** Clean up the logic, ensure no duplication, and check for "Newtype" opportunities.

> **Note on Cyclic Dependencies:** If an AI agent suggests a change that introduces a cycle, it must be rejected. Use **Inversion of Control** by defining a trait in the higher-level module that the lower-level module implements.
