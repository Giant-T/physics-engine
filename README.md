# MOTEUR PHYSIQUE

## Installation
Pour installer le projet il vous suffit d'installer [cargo/rust](https://rust-lang.org/tools/install/) sur votre ordinateur.

## Utilisation
Pour utiliser ce programme, vous pouvez simplement faire la commande suivante:
```bash
cargo run -r
```
Toutes les dépendances sont automatiquement installées.

Vous pouvez aussi simplement compiler le programme avec la commande suivante:
```bash
cargo build -r
```
Ne faites pas attention au multiple warnings, c'est principalement puisque
rust a des opinions fortes sur la nomenclature des variables
(snake_case tandis que parfois j'utilise des lettres majuscules pour faciliter la compréhension par rapport aux formules données dans le cours)

## Erreurs connues
- Si vous fermer le programme et vous utilisez Wayland, vous pourriez avoir un segfault, ça ne provient pas de mon code mais d'une de mes dépendances.

## TODO
- [ ] Investiguer le segfault lors de la fermeture du programme (Wayland/Smithay)
- [ ] Changer les noms de variables pour respecter les standards de Rust
- [ ] Optimiser linalg.rs (optionnel)
