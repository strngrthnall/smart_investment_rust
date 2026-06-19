# 📈 Smart Investment API (Rust)

[![Rust Version](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Framework](https://img.shields.io/badge/framework-Axum%20v0.8-blue.svg)](https://github.com/tokio-rs/axum)
[![Database](https://img.shields.io/badge/database-PostgreSQL-blue.svg)](https://www.postgresql.org/)
[![ORM/Query Builder](https://img.shields.io/badge/queries-SQLx%20v0.9%20(Compile%20Time%20Checked)-green.svg)](https://github.com/launchbadge/sqlx)

Uma API RESTful de alta performance e segurança de tipos para consolidação e gerenciamento de investimentos. Este projeto foi concebido seguindo princípios modernos de engenharia de software no ecossistema Rust, focando em segurança em tempo de compilação, baixo consumo de memória e concorrência segura.

---

## 🏗️ Arquitetura e Padrões de Projeto

A arquitetura do projeto foi estruturada para garantir o **desacoplamento**, a **testabilidade** e a **extensibilidade**, aplicando conceitos de arquiteturas limpas e em camadas:

```text
[ Cliente (Browser / HTTP Client) ]
               │
               ▼ (HTTP Request / JSON / Cookie)
┌──────────────────────────────────────────────────────────────┐
│                        Axum Router                           │
└──────────────┬──────────────────────────────┬────────────────┘
               │ (Extractors & Guards)        │ (Error propagation)
               ▼                              ▼
┌─────────────────────────────┐        ┌───────────────────────┐
│ Auth Layer                  │        │ AppError Layer        │
│ ─ Admin: Authorization      │        │ ─ Mapeia erros para   │
│ ─ User: Cookie JWT (HS256)  │        │   JSON / HTTP Status  │
└──────────────┬──────────────┘        └───────────────────────┘
               │
               ▼ (State & Repo injection)
┌──────────────────────────────────────────────────────────────┐
│                    Axum Handlers & Routes                    │
└──────────────┬──────────────────────────────┬────────────────┘
               │ (Askama Templates)           │ (Repository Trait)
               ▼                              ▼
┌─────────────────────────────┐        ┌───────────────────────┐
│ Server-Side Rendering (SSR) │        │ Repository Layer      │
│ ─ HTML Dinâmico (Askama)    │        │ ─ sqlx::PgPool        │
└─────────────────────────────┘        └──────────┬────────────┘
                                                  │
                                                  ▼ (Compile-time SQL)
                                       ┌───────────────────────┐
                                       │  PostgreSQL Database  │
                                       └───────────────────────┘
```

### Detalhamento das Camadas

1. **Camada de Apresentação & Roteamento (`src/routes/`)**
   * Controlada pelo **Axum**, um ecossistema assíncrono mantido pela equipe do Tokio.
   * Handlers puramente declarativos: cuidam do recebimento de requests JSON (para a API REST), da validação de payloads e do Server-Side Rendering (SSR) das páginas web utilizando templates **Askama**.

2. **Camada de Segurança & Guards (`src/auth/`)**
   * Utiliza os **Custom Extractors** (`FromRequestParts`) do Axum.
   * A autenticação e autorização são resolvidas diretamente nos handlers: a struct `Admin` valida requisições administrativas via header `Authorization`, enquanto a struct `User` decodifica tokens JWT contidos em Cookies `HttpOnly` para gerenciar sessões do frontend de forma nativa e segura.

3. **Camada de Acesso a Dados (`src/repository.rs`)**
   * Implementação do **Repository Pattern**. Isolamos o driver de banco de dados (`sqlx::PgPool`) dentro de um repositório dedicado.
   * O repositório também expõe um extrator do Axum, permitindo injeção de dependência simplificada diretamente nos handlers da API e do frontend.

4. **Garantia de Tipos SQL em Tempo de Compilação**
   * Integração com **SQLx**. Usando as macros de queries preparadas (ex: `sqlx::query_as!`), as consultas SQL são validadas contra o banco de dados rodando localmente durante o processo de build (`cargo build`). 
   * **Zero Runtime SQL Errors**: se houver erro de sintaxe ou incompatibilidade de tipo na tabela, o Rust previne a compilação.

5. **Tratamento de Erros Centralizado (`src/error.rs`)**
   * Tipagem estrita de erros usando a crate `thiserror` para modelar o enum `AppError`.
   * A integração com o trait `IntoResponse` do Axum garante que todas as falhas internas (banco de dados, autorização, renderização de templates, requisições HTTP) sejam traduzidas para formatos padronizados (JSON) e códigos de status HTTP corretos.

---

## 🛠️ Tecnologias Utilizadas

* **Rust (Edição 2024)**: O compilador mais moderno garantindo segurança de memória sem garbage collector.
* **Axum & Tokio**: Engine assíncrona multithread para APIs extremamente velozes com alto throughput e suporte a SSR.
* **SQLx**: Conector SQL assíncrono para PostgreSQL com checagem de tipos estática em tempo de compilação.
* **Askama**: Engine de templates HTML pré-compilada, fornecendo Server-Side Rendering rápido, seguro e verificado pelo compilador.
* **jwt-simple & password-auth**: Emissão/validação de tokens JWT e hashing de senhas seguro utilizando Argon2id.
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

## 🗺️ Funcionalidades Implementadas

O projeto encontra-se em seu estado de maturidade com as seguintes funcionalidades totalmente implementadas:

### 👤 1. Modelagem de Usuário e Autenticação
* **Domínio**: Entidade `User` com relações de carteiras de investimento individuais no banco PostgreSQL.
* **Segurança**: Criptografia de senhas utilizando a crate `password-auth` (Argon2id).
* **Migrações**: Tabelas de usuário (`users`) e de investimentos adquiridos (`owned_assets`) com chaves estrangeiras apropriadas, índices e regras de integridade relacional.

### 🔑 2. Sessões Seguras com JWT e Cookies
* **Sessões Stateless**: Geração e validação de JSON Web Tokens no login de usuários via crate `jwt-simple` (algoritmo HS256).
* **Cookies de Sessão**: Uso da crate `axum-extra` para definir cookies de sessão HTTP com a flag `HttpOnly` para mitigar ataques XSS.
* **Guards de Autenticação**: Extratores customizados do Axum (`User` e `Option<User>`) que decodificam o JWT dos cookies e autenticam requisições automaticamente nas rotas.

### 📊 3. Dashboard Dinâmico com Askama (SSR)
* **Server-Side Rendering (SSR)**: Interface web com renderização server-side para listagem de ativos disponíveis e carteira pessoal do investidor.
* **Askama Templates**: Templates HTML (`templates/`) pré-compilados pelo compilador Rust, garantindo validação de variáveis em tempo de compilação, segurança contra injeção de HTML e alto desempenho.

### 📈 4. Sincronização Automática com API Externa
* **Integração**: Conexão com a AwesomeAPI (`economia.awesomeapi.com.br`) para buscar cotações em tempo real de moedas e criptomoedas (Dólar, Bitcoin, Ethereum).
* **Otimização**: Mecanismo de throttling que atualiza os preços locais no banco de dados apenas se a última atualização for mais antiga que 24 horas.
* **Cálculo Delta**: Consolidação financeira dinâmica com cálculo de rentabilidade (Delta de Valor) efetuado diretamente na camada de banco de dados (PostgreSQL).
