# 📈 Smart Investment API (Rust)

[![Rust Version](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Framework](https://img.shields.io/badge/framework-Axum%20v0.8-blue.svg)](https://github.com/tokio-rs/axum)
[![Database](https://img.shields.io/badge/database-PostgreSQL-blue.svg)](https://www.postgresql.org/)
[![ORM/Query Builder](https://img.shields.io/badge/queries-SQLx%20v0.9%20(Compile%20Time%20Checked)-green.svg)](https://github.com/launchbadge/sqlx)

Uma API RESTful de alta performance e segurança de tipos para consolidação e gerenciamento de investimentos. Este projeto foi concebido seguindo princípios modernos de engenharia de software no[...]

---

## 🏗️ Arquitetura e Padrões de Projeto

A arquitetura do projeto foi estruturada para garantir o **desacoplamento**, a **testabilidade** e a **extensibilidade**, aplicando conceitos de arquiteturas limpas e em camadas


### Detalhamento das Camadas

1. **Camada de Apresentação & Roteamento (`src/routes/`)**
   * Controlada pelo **Axum**, um ecossistema assíncrono mantido pela equipe do Tokio.
   * Handlers puramente declarativos: cuidam apenas do recebimento de requests JSON, validação de payload e mapeamento das respostas.

2. **Camada de Segurança & Guards (`src/auth/`)**
   * Utiliza os **Custom Extractors** (`FromRequestParts`) do Axum.
   * Em vez de middleware genérico acoplado, a autenticação e autorização são resolvidas na própria assinatura dos handlers (como a struct `Admin`). Se o header falhar na extração, a rota[...]

3. **Camada de Acesso a Dados (`src/repository.rs`)**
   * Implementação do **Repository Pattern**. Isolamos o driver de banco de dados (`sqlx::PgPool`) dentro de um repositório dedicado.
   * O repositório também expõe um extrator do Axum, permitindo injeção de dependência simplificada diretamente nos handlers da API.

4. **Garantia de Tipos SQL em Tempo de Compilação**
   * Integração com **SQLx**. Usando as macros de queries preparadas (ex: `sqlx::query_as!`), as consultas SQL são validadas contra o banco de dados rodando localmente durante o processo de bui[...]
   * **Zero Runtime SQL Errors**: se houver erro de sintaxe ou incompatibilidade de tipo na tabela, o Rust previne a compilação.

5. **Tratamento de Erros Centralizado (`src/error.rs`)**
   * Tipagem estrita de erros usando a crate `thiserror` para modelar o enum `AppError`.
   * A integração com o trait `IntoResponse` do Axum garante que todas as falhas internas (banco de dados, autorização, recursos inexistentes) sejam traduzidas com consistência para formatos [...]

---

## 🛠️ Tecnologias Utilizadas

* **Rust (Edição 2024)**: O compilador mais moderno garantindo segurança de memória sem garbage collector.
* **Axum & Tokio**: Engine assíncrona multithread para APIs extremamente velozes com alto throughput.
* **SQLx**: Conector SQL assíncrono para PostgreSQL com checagem de tipos estática.
* **Tracing & Tracing-Subscriber**: Sistema de instrumentação e logging estruturado para diagnóstico em produção.
* **Dotenvy**: Carregamento dinâmico de variáveis de ambiente para separação de configurações.

---

## 🚀 Como Executar o Projeto

### Pré-requisitos
* Rust (MSRV 1.80+)
* Docker & Docker Compose
* SQLx CLI (`cargo install sqlx-cli --no-default-features --features postgres`)

### Passo a Passo

1. **Subir o Banco de Dados (PostgreSQL via Docker)**:
   ```bash
   docker-compose up -d
   ```

2. **Configurar o Ambiente**:
   Crie ou edite o arquivo `.env` na raiz do projeto:
   ```env
   DATABASE_URL=postgres://postgres:password123@localhost:5432/smart_investment
   ```

3. **Rodar as Migrações do Banco**:
   ```bash
   sqlx database setup
   # Ou execute as migrações diretamente:
   sqlx migrate run
   ```

4. **Executar a API**:
   ```bash
   cargo run
   ```
   A API estará rodando em `http://0.0.0.0:8000`.

### Executar Requisições de Teste
O projeto possui um diretório `http_requests/` com um arquivo `api.http` configurado para testes de integração local (usando extensões REST Client no VS Code ou JetBrains Gateway).

---