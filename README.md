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

## Référencement (SEO)

Chaque page publique déclare son titre, sa description et son adresse canonique via
`PageMeta` ([src/components/seo.rs]). Trois règles, qui ne sont pas optionnelles :

1. **Les balises doivent être rendues de façon synchrone.** `leptos_meta` n'injecte
   dans le `<head>` que ce qui a été rendu dans le **premier chunk** du flux SSR. Un
   `<Title>` posé dans un `Suspend` n'y arrive jamais — c'était le cas des pages
   d'atelier, qui partaient toutes avec le titre générique du site. Quand le texte
   dépend de données stockées, la ressource doit être une `Resource::new_blocking`,
   qui retient le premier chunk jusqu'à sa résolution. Même piège pour le statut HTTP :
   un `NotFound` dans un `Suspend` répondait 200.
2. **Rien de ce que `PageMeta` pose ne doit l'être aussi globalement.** `<Meta>` et
   `<Link>` s'accumulent sans dédoublonnage : une balise posée ici *et* dans
   [src/app.rs] apparaîtrait deux fois. `app.rs` ne garde que `og:site_name`,
   `og:type` et `og:locale`, qu'aucune page ne redéfinit. `<Title>` fait exception —
   c'est un emplacement unique où le dernier écrivain gagne.
3. **L'adresse du site est une constante**, `SITE_ORIGIN` dans
   [src/models/studio.rs]. Pas une variable d'environnement : `app.rs` est compilé
   aussi pour le WASM, où `std::env::var` répond toujours `Err`. Le jour où un domaine
   propre est attaché, cette ligne est la seule à changer — plus une redirection 301
   depuis l'ancien hôte, à poser côté Cloudflare.

`/robots.txt` et `/sitemap.xml` sont des routes actix ([src/seo.rs]), pas des fichiers
d'`assets/` : ce répertoire est monté sous `/assets`, où aucun robot ne va les chercher.
Le sitemap est reconstruit à chaque requête depuis la base, et liste chaque atelier sous
le seul préfixe de sa section.

**Ce qui reste à faire, et qui ne passe pas par le code :** pour une activité locale, une
**fiche Google Business Profile** vérifiée (mêmes coordonnées que `OWNER`, horaires,
photos, lien vers le site) pèse plus lourd que tout ce qui précède. Et le site est à
déposer dans la **Google Search Console**, où le sitemap se soumet.

Le texte de `/moi-et-mon-atelier/mon-atelier` est un brouillon tiré de ce que le site
dit déjà de lui-même : il n'affirme rien de neuf sur le lieu, faute de le savoir. À
réécrire.

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


