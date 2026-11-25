# Correcao da animacao da intro (Bink) - port de ScarlettPort

## Sintoma
Mesmo com 30 fps, a animacao da Rockstar (murais, aguia, estrelas) roda travada,
com cortes: "one old picture after another on the screen".

## Causa (documentada no ScarlettPort)
`refs/ScarlettPort/runtime/victory/crates/victory-gpu/src/texture.rs:795-827`:

> A texture that changed without the guard seeing it is hashed at every check
> for a while (the intro's movie: three pictures in turn that only changed
> every eighth frame, one old picture after another on the screen).

O texture refresh pula o hash em 7 de cada 8 frames (`hash_every=8`). Os planos
do video Bink sao texturas lineares de 1 mip; quando a escrita do decoder nao
e vista pelo guard, o jogo apresenta a imagem antiga por varios frames.

## Correcao
```rust
let film = desc.swizzle.0 == 0 && desc.mips == 1;
if unwritten && !turn && !film && frame >= last.unguarded {
    return 0;  // pula o hash - ERRADO para filmes
}
```
Ou seja: **texturas lineares de 1 mip (planos de filme) nunca pulam o hash**;
sao verificadas em todo check, e o conteudo muda a cada frame do video.

## Estado no gta5700
- Detectado em 25 Nov; ainda nao portado para `repos/victory-fork` (a base do
  fork e `a803c87`, mais antiga, sem `VICTORY_TEXTURE_GUARD`/`memcopy::hash_every`).
- Port candidates:
  1. `VICTORY_TEXTURE_GUARD=0` (se existir na nossa base): forca hash em todo check.
  2. Portar o `film` check e `unseen_write` do ScarlettPort para `texture.rs`.
  3. Alternativa diagnostics: `VICTORY_ALWAYS_COPY=1` do fork Belobug force
     recopiar toda copy de memoria do titulo a cada frame - confirma se o
     problema e stale copy.
- Ver tambem `descriptor/host.rs::RECHECK_FRAMES=[4,12,40,120,400]`: o primeiro
  recheck apos upload e 4 frames, lento para video a 30 fps.

## Pseudocodigo para implementar
```
ao refrescar uma textura:
    film = linear swizzle && mip_levels == 1
    if filme e sem escrita vista pelo write-watch:
        fazer hash mesmo assim (nao pular)
        se hash mudou: reenviar textura neste frame
    else:
        seguir a logica de turn/hash_every atual
```
