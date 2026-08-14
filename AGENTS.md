# Agent Instructions — Stemway Project

## Workflow Rules

### Pesquisa de Documentação Obrigatória

Antes de iniciar qualquer nova fase ou ao encontrar **≥2 erros de compilation consecutivos**:

1. Pesquisar a **documentação oficial atualizada** das crates/libraries em uso
2. Verificar a **versão exata** declarada no `Cargo.toml` e buscar docs correspondentes (não usar a versão mais recente, mas a versão específica do projeto)
3. Consultar exemplos reais no [docs.rs](https://docs.rs), [crates.io](https://crates.io), ou repositórios oficiais
4. Validar a API antes de escrever código — `cargo doc --open` local ou docs.rs online

**Motivo:** APIs mudam entre versões. A Fase 2 gerou 15+ minutos de debugging que poderiam ter sido evitados verificando a API do Symphonia 0.5.5 antes de implementar.

### Commits

- Prefixos convencionais: `feat:`, `fix:`, `refactor:`, `docs:`, `chore:`, `test:`
- **Fazer commit ao final de cada fase concluída** com mensagem descritiva (ex: `feat: Phase 3 - ONNX inference + chunk processor`)
- Merge squash ao final de cada fase
- Nunca commit secrets ou changes em `.env`

### Dependências

- Sempre verificar se uma crate já está declarada antes de adicionar
- Verificar compatibilidade de versão com o projeto antes de atualizar
