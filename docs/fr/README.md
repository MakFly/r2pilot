# Documentation r2pilot

> Français | [English](../en/README.md)

## Table des matières

- [Vue d'ensemble](#vue-densemble)
- [Installation](#installation)
- [Configuration](#configuration)
- [Commandes](#commandes)
- [Exemples](#exemples)

## Vue d'ensemble

**r2pilot** est un outil CLI en Rust pour gérer le stockage Cloudflare R2 depuis votre terminal.

### Fonctionnalités

- **Gestion des buckets** : Lister, créer, supprimer des buckets
- **Opérations sur les fichiers** : Upload, téléchargement, suppression
- **URLs signées** : Générer des URLs présignées pour un accès temporaire
- **Configuration interactive** : Assistant de configuration guidé
- **Authentification multiple** : Support des API Tokens et Access Keys

## Installation

### Depuis le code source

```bash
# Cloner le dépôt
git clone https://github.com/MakFly/r2pilot.git
cd r2pilot

# Build release
cargo build --release

# Créer un lien symbolique
ln -s $(pwd)/target/release/r2pilot ~/bin/r2pilot

# Recharger votre shell
source ~/.zshrc  # ou source ~/.bashrc
```

### Prérequis

- Rust 1.70+ (pour compiler depuis le source)
- Compte Cloudflare avec R2 activé

## Configuration

### Configuration initiale

Lancez l'assistant interactif :

```bash
r2pilot init
```

Vous aurez besoin de :
1. Votre ID de compte Cloudflare (32 caractères alphanumériques)
2. Méthode d'authentification :
   - **API Token** (recommandé pour les opérations API Cloudflare)
   - **Access Key ID + Secret Access Key** (requis pour les opérations compatibles S3)
3. Le nom de votre bucket R2 par défaut

### Fichier de configuration

La configuration est stockée dans `~/.config/r2pilot/config.toml` :

```toml
[cloudflare]
account_id = "votre_id_de_compte"
endpoint = "https://votre_id_de_compte.r2.cloudflarestorage.com"
api_token = "votre_api_token"  # OU access_key_id + secret_access_key
access_key_id = "votre_access_key_id"
secret_access_key = "votre_secret_access_key"

[r2]
default_bucket = "nom_de_votre_bucket"
region = "auto"
default_expiration = 7200  # 2 heures en secondes
```

### Obtenir vos identifiants

**API Token** (pour la gestion des buckets) :
1. Allez sur https://dash.cloudflare.com/profile/api-tokens
2. Créez un token avec les permissions R2 Edit

**Access Keys** (pour les opérations sur fichiers) :
1. Allez sur https://dash.cloudflare.com/<account_id>/r2/api-tokens
2. Gérez les API Tokens R2 pour votre bucket
3. Récupérez votre Access Key ID et Secret Access Key

## Commandes

### init

Initialiser la configuration avec l'assistant interactif.

```bash
r2pilot init
```

### config

Gérer la configuration.

```bash
# Afficher la configuration actuelle
r2pilot config show

# Éditer la configuration dans $EDITOR
r2pilot config edit

# Valider les credentials et tester la connexion
r2pilot config validate
```

### tokens

Gérer les API tokens Cloudflare.

```bash
# Lister tous les API tokens
r2pilot tokens list

# Créer un nouveau token R2
r2pilot tokens create

# Révoquer un token
r2pilot tokens revoke <token_id>
```

### buckets

Gérer les buckets R2.

```bash
# Lister tous les buckets
r2pilot buckets list

# Créer un nouveau bucket
r2pilot buckets create mon-bucket

# Supprimer un bucket
r2pilot buckets delete mon-bucket

# Informations sur un bucket
r2pilot buckets info mon-bucket

# Lister le contenu d'un bucket
r2pilot buckets ls mon-bucket
```

### files

Gérer les fichiers dans R2.

```bash
# Upload un fichier
r2pilot files upload fichier-local.txt chemin/distant.txt --bucket mon-bucket --progress

# Télécharger un fichier
r2pilot files download chemin/distant.txt fichier-local.txt --bucket mon-bucket

# Supprimer un fichier
r2pilot files delete chemin/distant.txt --bucket mon-bucket

# Lister les fichiers
r2pilot files ls --prefix chemin/vers/
```

### urls

Générer des URLs signées.

```bash
# Générer une URL signée (défaut: 2 heures)
r2pilot urls generate chemin/vers/fichier.txt

# Expiration personnalisée (en secondes)
r2pilot urls generate chemin/vers/fichier.txt --expires 3600

# Sortie JSON
r2pilot urls generate chemin/vers/fichier.txt --output json
```

### completion

Générer les scripts de complétion de shell.

```bash
# Générer pour bash
r2pilot completion bash

# Générer pour zsh
r2pilot completion zsh

# Générer pour fish
r2pilot completion fish
```

### doctor

Diagnostics et dépannage.

```bash
# Vérifier l'installation
r2pilot doctor check

# Tester la connexion R2
r2pilot doctor test-connection
```

## Exemples

### Première configuration

```bash
# Lancer l'assistant de configuration
r2pilot init

# Valider votre configuration
r2pilot config validate
```

### Upload une vidéo

```bash
# Upload vers R2 avec barre de progression
r2pilot files upload video.mp4 raw/video-2024-01.mp4 --progress

# Upload vers un bucket spécifique
r2pilot files upload video.mp4 videos/january.mp4 --bucket mes-videos
```

### Générer un lien partageable

```bash
# Générer un lien valide 2 heures (défaut)
r2pilot urls generate videos/january.mp4

# Générer un lien valide 1 heure
r2pilot urls generate videos/january.mp4 --expires 3600

# Obtenir une sortie JSON pour les scripts
r2pilot urls generate videos/january.mp4 --output json
```

### Lister et gérer les buckets

```bash
# Lister tous les buckets
r2pilot buckets list

# Lister le contenu d'un bucket
r2pilot buckets ls mon-bucket

# Créer un nouveau bucket
r2pilot buckets create backups

# Supprimer un bucket (pas celui par défaut)
r2pilot buckets delete ancien-bucket
```

### Dépannage

```bash
# Vérifier si r2pilot est installé correctement
r2pilot doctor check

# Tester la connexion R2
r2pilot doctor test-connection

# Afficher la configuration actuelle
r2pilot config show
```

## Conseils

- **Bucket par défaut** : Définissez un bucket par défaut pour éviter de spécifier `--bucket` à chaque fois
- **Barre de progression** : Utilisez le flag `--progress` pour les uploads de fichiers volumineux
- **Sortie JSON** : Utilisez `--output json` pour les scripts et l'automatisation
- **Complétion de shell** : Activez la complétion pour une meilleure expérience de commande

## Dépannage

### "Access Key ID non configuré"

Lancez `r2pilot init` pour configurer vos identifiants, ou éditez manuellement `~/.config/r2pilot/config.toml`.

### "API Token requis pour cette opération"

Certaines opérations nécessitent un API Token avec les permissions R2 :
- Gestion des buckets (list, create, delete)
- Gestion des tokens

Obtenez votre API Token depuis : https://dash.cloudflare.com/profile/api-tokens

### "Access Keys requises pour cette opération"

Les opérations sur fichiers (upload, download) nécessitent les R2 Access Keys :
1. Allez sur votre dashboard R2
2. Naviguez vers API Tokens
3. Récupérez votre Access Key ID et Secret Access Key

## Licence

MIT - Voir [LICENSE](../../LICENSE) pour les détails.
