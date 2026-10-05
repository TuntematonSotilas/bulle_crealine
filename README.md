# bulle_crealine

Bulle Crealine

Live here : https://bulle-crealine.onrender.com

## Setup
 
* Install Cargo Leptos : `cargo install --locked cargo-leptos`
* Install Tailwind :  `npm i`

## Run 

    cargo leptos watch

## Build

    cargo leptos build --release

## Lint 

    cargo clippy
    
## Tests

    cargo test --features ssr

## Réservations (MongoDB)

Deux collections dans la base `bulle_crealine_db` :

| Collection | Contenu |
| --- | --- |
| `sessions` | les séances proposées : type d'atelier, date, thème, prix, nombre de places |
| `bookings` | les réservations : séance, nom, e-mail, téléphone, nb de personnes, commentaire du client, note interne de l'admin |

Une seule variable est nécessaire :

| Variable | Rôle |
| --- | --- |
| `MONGODB_URI` | chaîne de connexion du cluster |
| `MONGODB_DATABASE` | facultatif, remplace `bulle_crealine_db` |

Sans `MONGODB_URI`, le site public démarre normalement : seules les pages de
réservation et d'administration signalent que les données sont inaccessibles.

    $env:MONGODB_URI = "mongodb+srv://..."
    cargo leptos watch

## Mentions légales : ce qu'il reste à compléter

`/mentions-legales` est servie et complète, **à deux mentions près** : le SIRET et le
statut TVA portent `[À COMPLÉTER]` et s'affichent tels quels. Tant que le SIRET manque,
**la page ne satisfait pas l'article 6-III de la LCEN.**

Les deux se remplissent au même endroit, dans `OWNER` ([src/models/studio.rs]) — une
chaîne chacun, rien d'autre à toucher. Pour la TVA, c'est soit le numéro
intracommunautaire, soit la phrase « TVA non applicable, article 293 B du CGI », jamais
le vide.

La page annonce par ailleurs une conservation des réservations de **3 ans après le
dernier contact**. Rien ne l'applique aujourd'hui : `booking::delete` est un effacement
logique, le document reste. La promesse est donc faite et pas encore tenue.

## Administration

Il n'y a qu'un seul compte admin, défini par trois variables d'environnement — pas de
base d'utilisateurs, pas d'inscription. Sans ces variables, le site public
démarre normalement et `/admin` reste inaccessible.

| Variable | Rôle |
| --- | --- |
| `ADMIN_EMAIL` | Adresse acceptée à la connexion |
| `ADMIN_PASSWORD_HASH` | Hash Argon2 du mot de passe, au format PHC |
| `ADMIN_SESSION_SECRET` | Clé de signature des cookies, 32 caractères minimum |

Les deux dernières se génèrent d'un coup :

    cargo run --example hash_password --features ssr -- "<mot de passe>"

En local (.bashrc) :

    export ADMIN_EMAIL="vous@exemple.fr"
    export ADMIN_SESSION_SECRET="<secret>"
    export ADMIN_PASSWORD_HASH="<hash>"

### Fonctionnement

La connexion vérifie le mot de passe avec Argon2, puis dépose un cookie signé en
HMAC-SHA256 contenant l'adresse et une date d'expiration. Ce cookie est
`HttpOnly` (invisible au JavaScript, donc hors de portée d'une XSS) et
`SameSite=Lax` (non envoyé sur une requête venue d'un autre site, ce qui écarte
la falsification de requête). Il est revérifié à chaque requête, sans aucun état
côté serveur.

En contrepartie de ce format auto-porté, **un cookie ne peut pas être révoqué à
distance** avant son expiration, fixée à 8 h. Pour invalider immédiatement tous
les accès en circulation, changez `ADMIN_SESSION_SECRET` puis redémarrez.

Les tentatives ratées sont limitées à 5 par adresse IP, puis bloquées 15 min.
L'adresse venant des en-têtes du proxy, ce compteur gêne une attaque naïve mais
ne résiste pas à quelqu'un qui fait tourner l'adresse annoncée : la vraie
protection reste la longueur du mot de passe, qu'Argon2 rend coûteux à deviner.

### Ajouter des pages ou des données

`admin_guard` protège les pages `/admin/*`, mais **pas** les server functions,
servies sous `/api`. Toute server function réservée à l'administration doit donc
commencer par :

```rust
let admin = crate::auth::require_admin()?;
```

## Docker

* Build : `docker build . -t bulle_crealine`
* Run : `docker run -p 3000:8080 bulle_crealine`
* Test : http://localhost:3000


## Docs

* Leptos : https://leptos.dev
* Rust-UI :  https://rust-ui.com


