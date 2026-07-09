# Workflow

## Documentação automática
Ao escrever ou modificar código, adicione docstrings (`///`) em toda
função, struct, enum e módulo público que criar ou alterar, explicando
propósito, fluxo e lógica.

- Idiomatic Rust: prefira ownership, borrowing e lifetime explícitos. Evite unwrap() em tipos Result/Option — trate erros propagando ou com fallback.

- Concorrência: justifique o uso de Arc, Mutex ou Rc — não use por padrão.

- Clone: evite clone() desnecessário; prefira referências (&T) quando possível.

- Iteração: após cada alteração, rode cargo check e corrija os erros do compilador.