# Study Bot

An extremely simple study bot made using the Poise framework.

## Commands

- /profile <optional user>
  - Gets the amount of time the user has spent studying. If user parameter is empty, the command will target the sender of the command.
- /leaderboard
  - Gets the top 10 users in terms of total study time.
- /add_study_channel <voice channel>
  - Adds a voice channel to the list of study channels.
- /remove_study_channel <voice channel>
  - Removes a voice channel from the list of study channels.

## Setup

First, create a .env file at the root directory of the repository. Fill in its contents as such:

```
DISCORD_TOKEN=<your discord bot token>
```

Of course, substitute `<your discord bot token>` for your actual discord bot token.
The bot will generate a `data.db` file in its directory (root of the repository), so make sure has the permissions do that.
