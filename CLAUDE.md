# Kontro

## Branches

- Toda tarefa sai da `develop`, nunca da `main`. Antes de começar: `git fetch origin develop`
  e `git checkout -b <tema>/<tarefas> origin/develop` (ex.: `fluidez/t36-t39`,
  `ci/beta-sem-buraco`).
- O PR vai para a `develop`. A `main` só recebe a `develop` na hora de lançar uma versão
  (ver "Publicando uma versão" no `README.md`).
- Se a sessão já começou em outra branch, crie a de trabalho a partir de `origin/develop`
  antes do primeiro commit.

## Tarefas

- A lista do que construir, em ordem e com critério de pronto, está no `PLANO.md`; o
  registro do que quebrou e por quê, no `TAREFAS.md`.
- Antes de pegar tarefas, leia a seção **Situação** do `PLANO.md` **na `develop`** e a lista
  de PRs abertos: é lá que está o que já foi entregue. A `main` fica atrás até o próximo
  lançamento.
- Ao entregar, atualize a **Situação** do `PLANO.md` no mesmo PR, e registre ali qualquer
  desvio do que a tarefa pedia, com o motivo.

## Convenções

As do `README.md`, seção "Convenções": tudo em português, sem comentários no código, todo
seletor CSS começa pela raiz da própria folha, `cargo fmt` com o `rustfmt.toml`.
