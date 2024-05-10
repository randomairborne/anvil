# Experienced

A powerful, flexible discord leveling bot

## How to Self-Host Experienced

The easisest way to self-host experienced is on Debian or Ubuntu Linux. Thus, this is how this tutorial will set up
Experienced.

If you'd rather just have a hosted bot, that's fine! I really appreciate
it. [Click here to invite the official instance.](https://discord.com/api/oauth2/authorize?client_id=1035970092284002384&permissions=0&scope=bot%20applications.commands)

### Creating a Discord application

First, we need to create a Discord application. Go
to [https://discord.com/developers/applications](https://discord.com/developers/applications) and click
the `New Application` button in the top right corner.
Give it a nice name, then click continue. Now we need to create a .env file, which should look like this:

```dotenv
DATABASE_URL=
DISCORD_TOKEN=
REDIS_URL=
CONTROL_GUILD=
OWNERS=
ROOT_URL=
```

Go to the `Bot` tab. This will show you a `Reset Token` button. Clicking this should reveal and copy your bot token,
which should then be filled into the `DISCORD_TOKEN`. Then, customize your bot to your heart's content. No gateway
intents are needed.
While you are legally within your rights to do so, please do not self-host public instances of Experienced.

### Preparing your server

To run experienced, you need [docker](https://docs.docker.com/engine/install/)
or [podman](https://podman.io/docs/installation). We'll use docker for this tutorial.
You also need postgres and redis. You can get these by running

```bash
sudo apt install redis-server postgresql-16
```

Then, you can create a new user with

```bash
sudo su postgres
psql -U postgres -c "CREATE USER xpd PASSWORD 'xpd'"
psql -U postgres -c "CREATE DATABASE xpd OWNER xpd"
exit
```

edit the .env file, filling in `<things in angle brackets>` with your text.

```dotenv
DISCORD_TOKEN=<your discord bot token>
DATABASE_URL=postgres://xpd:xpd@host.docker.internal/xpd
REDIS_URL=redis://host.docker.internal:6379
CONTROL_GUILD=<the server id of a server you will control the bot from>
OWNERS=<your discord ID>
ROOT_URL=<your website url>
```

### Starting the bot

Finally, you can actually run the bot!

```bash
docker run --env-file .env --add-host=host.docker.internal:host-gateway --detach ghcr.io/randomairborne/xpd-gateway:latest
```

And you're done! This mini-tutorial doesn't support the leaderboard, though. You also may wish to set up a simple
docker compose file. [Join the discord](https://valk.sh/discord) to learn more!


# Using Docker Compose
You can copy the variables below, or grab them from [here](<https://github.com/PBOwner/xpd/blob/prod/docker-compose.yml>)

## Docker Compose File:

```bash
networks:
  xpd-network:
    external: false

services:
  xpd-gateway:
    image: ghcr.io/randomairborne/xpd-gateway:latest
    restart: unless-stopped
    networks:
      - xpd-network
    env_file:
      - .env
    environment:
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis
      - postgres

  redis:
    image: redis
    restart: unless-stopped
    networks:
      - xpd-network

  postgres:
    image: postgres:16-alpine3.19
    restart: unless-stopped
    networks:
      - xpd-network
    env_file:
      - .env
    environment:
      - POSTGRES_USER=xpd
      - POSTGRES_DB=xpd
    volumes:
      - ./postgres:/var/lib/postgresql/data
```
Just Copy and Paste this into a `docker-compose.yml` file.

## Env File:
You can copy the variables below, or grab them from [here](<https://github.com/PBOwner/xpd/blob/prod/.env.example>)
```bash
POSTGRES_PASSWORD=<db_pass>
DATABASE_URL=postgres://xpd:<db_pass>@postgres:5432/xpd
DISCORD_TOKEN=<token>
CONTROL_GUILD=<main_discord_server_id>
OWNERS=<your_discord_id>
ROOT_URL=https://my.domain.com
```

Make sure you replace `<token>` and both `<db_pass>` with your own bot token and database password for postgres. You do **NOT** need to configure postgres, just set those variables up and it will configure for you.

## Invite the bot with:

```bash https://discord.com/oauth2/authorize?client_id=<yourclientid>&permissions=8&scope=bot+applications.commands```

Make sure to replace the `<yourclientid>` with your own.
