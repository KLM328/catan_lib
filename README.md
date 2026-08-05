# catan_lib

Moteur de jeu Catan écrit en Rust, sous forme de bibliothèque pure : aucune interface graphique, aucun réseau, aucune dépendance à un environnement d'exécution particulier.

La bibliothèque est **déterministe** : à état égal et action égale, elle produit toujours le même résultat. Tout le hasard (dés, mélange des tuiles, carte volée) est fourni par l'appelant. Cette propriété est volontaire — elle permet de faire tourner le même moteur sur un serveur et sur plusieurs clients sans divergence, et de rejouer une partie depuis son journal.

## État du projet

**Version 0.1.0** — une partie complète est jouable de bout en bout, de la mise en place à la détection de la victoire.

### Implémenté

- Génération de la topologie hexagonale (tuiles, sommets, arêtes et leurs adjacences), pour n'importe quelle forme de plateau
- Mise en place du plateau standard : répartition des terrains, pose des jetons en escargot, position initiale du voleur
- Détermination de l'ordre des joueurs par lancer de dés
- Phase de placement initial en serpentin (2 colonies + 2 routes par joueur), avec crédit des ressources de la deuxième colonie
- Production sur jet de dés, voleur inclus
- Construction de routes, de colonies, amélioration en villes, avec paiement atomique
- Résolution du 7 : défausses, déplacement du voleur, vol d'une carte
- Tour par tour, points de victoire, fin de partie

### Non implémenté

- Cartes développement
- Ports et échanges (entre joueurs comme avec la banque)
- Banque à réserve finie (les ressources sont actuellement créées sans limite)
- Route la plus longue et armée la plus grande
- Plateaux autres que le standard (l'architecture les accepte, seules les données manquent)
- Sérialisation (`serde`)

## Utilisation

```rust
use catan_lib::{Game, Player, PlayerColor, Roll, Scenario};

// 1. Créer la partie (2 à 6 joueurs)
let scenario = Scenario::standard();
let terrains = scenario.terrains().to_vec();
let mut game = Game::new(scenario, vec![
    Player::new(PlayerColor::Blue),
    Player::new(PlayerColor::Red),
])?;

// 2. Déterminer l'ordre des joueurs
//    Les jets sont fournis par l'appelant : Roll::random() côté serveur,
//    valeurs fixes en test.
game.set_players_order(vec![Roll::random(), Roll::random()])?;

// 3. Installer le plateau
//    `terrains` est l'agencement des tuiles. Le mélange se fait à l'extérieur ;
//    le moteur se contente de valider qu'il correspond bien au scénario.
game.start(&terrains)?;

// 4. Phase de placement : chaque joueur pose une colonie puis une route.
//    Le tour avance automatiquement, le statut indique ce qui est attendu.
game.build_settlement(game.current_player(), vertex)?;
game.build_road(game.current_player(), edge)?;

// 5. Phase de jeu
game.apply_roll(Roll::random())?;
game.build_road(game.current_player(), edge)?;
game.next_player()?;
```

### Le statut pilote tout

`GameStatus` indique en permanence ce que la partie attend. Chaque action vérifie le statut et refuse les coups hors séquence, ce qui rend impossible de lancer les dés deux fois, de poser deux colonies d'affilée pendant la mise en place, ou de construire avant d'avoir lancé.

| Statut | Attendu |
|---|---|
| `Starting` | `start()` |
| `FirstPlacementSettlement` / `SecondPlacementSettlement` | `build_settlement()` |
| `FirstPlacementRoad` / `SecondPlacementRoad` | `build_road()` |
| `AwaitingRoll` | `apply_roll()` |
| `AwaitingDiscard { must_discard }` | `discard()` pour chaque joueur concerné |
| `AwaitingNewRobberLocation` | `move_robber()` |
| `AwaitingSteal` | `steal()` |
| `PlayingActions` | constructions, puis `next_player()` |
| `End { winner }` | la partie est terminée |

### Le hasard reste à l'extérieur

Le moteur ne tire jamais au sort. Trois points d'entrée reçoivent l'aléatoire :

- `set_players_order(rolls)` et `apply_roll(roll)` reçoivent les jets de dés
- `start(arrangement)` reçoit l'agencement des tuiles, mélangé par l'appelant
- `steal(player, Some(steal))` reçoit la carte volée

En pratique, le serveur tire, applique, puis transmet le résultat aux clients, qui rejouent exactement le même calcul.

## Architecture

```
src/
├── geometry/     Hex, Topology, TileId, VertexId, EdgeId  (aucune dépendance)
├── resource/     Resource, ResourceCounts, Hand, Cost
├── roll.rs       Roll
├── player.rs     Player, PlayerId
├── board/        Board, Tile, Terrain, Building, Production
├── scenario.rs   Scenario  (définition d'une variante de plateau)
└── game.rs       Game      (orchestration, tours, phases)
```

Les dépendances entre modules descendent toujours : `game` → `scenario` → `board` → `geometry` / `resource`. Aucun cycle.

Deux séparations structurent le reste :

**Topologie et état.** `Topology` contient la géométrie, figée à la création. `Board` contient ce qui change : tuiles, bâtiments, routes, voleur. Les sommets et arêtes ne sont jamais des objets mais des indices (`VertexId`, `EdgeId`), ce qui évite tout graphe de références et garde `Board` clonable et sérialisable.

**Vérifier puis appliquer.** Chaque règle existe en deux temps : une fonction pure qui valide et calcule (`can_place_road`, `production`), et une fonction courte qui applique (`place_road`, `apply_roll`). Une action refusée ne modifie jamais l'état — un paiement impossible ne prélève rien.

## Développement

```bash
cargo test      # 61 tests
cargo clippy
```

La suite de tests se répartit sur deux niveaux : les règles sont testées unitairement au niveau de `Board` et `Topology` (sur des plateaux minimaux construits à la main), tandis que `game.rs` contient un test d'intégration qui joue une partie réelle du début à la fin, sans jamais forcer un état.

Prérequis : Rust 1.85 ou plus récent (édition 2024).