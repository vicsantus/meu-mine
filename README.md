# Meu Mine

Um clássico jogo de Campo Minado (*Minesweeper*) desenvolvido em Rust com [Macroquad](https://macroquad.rs), compatível com Desktop e Android.

O projeto inclui um gerador de tabuleiros sem adivinhação (*no-guess*), garantindo que as partidas possam ser resolvidas usando pura dedução lógica a partir do primeiro clique.

## Funcionalidades

- **Gerador sem adivinhação (*No-Guess*)**: Tabuleiros validados por um solver lógico (inspirado no algoritmo de Simon Tatham's Portable Puzzle Collection), eliminando situações de 50/50.
- **Primeiro clique seguro**: O primeiro clique sempre abre uma área segura sem minas ao redor.
- **Níveis de dificuldade clássicos**:
  - **Fácil**: 9x9 com 10 minas
  - **Médio**: 16x16 com 40 minas
  - **Difícil**: 30x16 com 99 minas
  - **Personalizado**: Dimensões de 9x9 até 50x50 com quantidade customizada de minas.
- **Controles e mecânicas completas**:
  - Abertura de células e *flood fill* em áreas vazias.
  - Marcação de bandeiras e pontos de interrogação (botão direito ou modo "Bandeira" na interface para telas touch).
  - *Chord* (abertura rápida): clicar em um número já aberto com as bandeiras corretas ao redor abre os vizinhos restantes.
  - Carinha clássica com reações de status (normal, surpreso, vitorioso e derrotado) e reinício rápido.
  - Contadores no estilo display digital para minas restantes e cronômetro.
- **Interface responsiva**: Escalonamento automático da interface para diferentes resoluções e telas móveis.

## Como Executar

### Pré-requisitos

- [Rust](https://www.rust-lang.org/) (edição 2021)

### Desktop

Para rodar diretamente no desktop:

```bash
cargo run --release
```

Para executar a suíte de testes:

```bash
cargo test
```

### Android (APK)

O projeto possui um script para geração do APK via Docker usando `cargo-quad-apk`:

```bash
./build-apk.sh
```

O arquivo gerado ficará disponível em `target/android-artifacts/release/apk/meu-mine.apk`.

## Estrutura do Código

- `src/main.rs`: Interface gráfica, renderização imediata com Macroquad, HUD, controles de entrada e modal de configuração.
- `src/game.rs`: Regras de negócio do jogo (estados das células, contagem de minas, vitória/derrota, *chord* e *flood fill*).
- `src/generator.rs`: Gerador de tabuleiros e solver lógico para validação de partidas sem adivinhação.