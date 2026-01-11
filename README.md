# r2pilot 🚀

> CLI Rust pour gérer Cloudflare R2 depuis votre terminal

📖 **Documentation** : [Français](docs/fr/README.md) | [English](docs/en/README.md)

## Installation

```bash
# Clone le dépôt
git clone https://github.com/MakFly/r2pilot.git
cd r2pilot

# Build release
cargo build --release

# Créer un lien symbolique
ln -s $(pwd)/target/release/r2pilot ~/bin/r2pilot

# IMPORTANT : Recharger votre session pour que le PATH soit pris en compte
# Option 1 : Recharger votre shell
source ~/.zshrc
# Option 2 : Fermer et rouvrir votre terminal
# Option 3 : Utiliser le chemin complet : ~/bin/r2pilot
```

## Utilisation rapide

```bash
# Voir l'aide
r2pilot --help

# Configuration initiale (wizard interactif)
r2pilot init

# Lister les buckets
r2pilot buckets list

# Uploader un fichier
r2pilot files upload video.mp4 raw/video123.mp4 --bucket my-bucket --progress

# Générer une URL signée
r2pilot urls generate path/to/file.mp4 --expires 7200
```

## Commandes

| Commande | Description |
|----------|-------------|
| `init` | Configuration initiale (wizard interactif) |
| `config` | Gestion de la configuration |
| `tokens` | Gestion des API tokens Cloudflare |
| `buckets` | Gestion des buckets R2 |
| `files` | Gestion des fichiers (upload, download, delete, ls) |
| `urls` | Génération d'URLs signées (GET, PUT, DELETE) |
| `cors` | Gestion CORS (interactive ou JSON) |
| `lifecycle` | Règles de cycle de vie (interactive ou JSON) |
| `website` | Hébergement statique (public bucket) |
| `completion` | Shell completion |
| `doctor` | Diagnostic et vérification |

## Configuration

La configuration est stockée dans `~/.config/r2pilot/config.toml` :

```toml
[cloudflare]
account_id = "your_account_id"
api_token = "your_api_token"  # OU access_key_id + secret_access_key
endpoint = "https://your_account_id.r2.cloudflarestorage.com"

[r2]
default_bucket = "your_bucket_name"
default_expiration = 7200
```

## Documentation

Pour une documentation complète, consultez :

- 🇫🇷 **[Documentation française](docs/fr/README.md)** - Guide complet en français
- 🇬🇧 **[English Documentation](docs/en/README.md)** - Full documentation in English

## Exemples d'utilisation

```bash
# Uploader un fichier (avec barre de progression)
r2pilot files upload video.mp4 videos/jan.mp4 --progress

# Uploader un gros fichier en multipart (automatique >100MB)
r2pilot files upload largefile.iso backups/large.iso --progress

# Télécharger un fichier
r2pilot files download videos/jan.mp4 video.mp4

# Lister les fichiers d'un bucket
r2pilot buckets ls my-bucket

# Générer une URL signée (GET, par défaut 2h)
r2pilot urls generate videos/jan.mp4 --expires 3600

# Générer une URL signée pour upload (PUT)
r2pilot urls generate videos/new.mp4 --method put --expires 3600 --content-type video/mp4

# Générer une URL signée pour suppression (DELETE)
r2pilot urls generate videos/old.mp4 --method delete --expires 3600

# Configurer CORS (mode interactif)
r2pilot cors set --interactive

# Configurer CORS (fichier JSON)
r2pilot cors set --file cors.json

# Voir la configuration CORS
r2pilot cors get

# Supprimer la configuration CORS
r2pilot cors delete

# Configurer les règles de cycle de vie (mode interactif)
r2pilot lifecycle set --interactive

# Activer l'hébergement statique (public bucket)
r2pilot website enable --index index.html --error 404.html

# Voir la configuration website
r2pilot website get

# Désactiver l'hébergement statique
r2pilot website disable
```

## License

MIT
