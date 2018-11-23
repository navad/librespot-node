# librespot-node

An easy to use Node.js wrapper for [librespot](https://github.com/librespot-org/librespot), an open source Spotify client, based on [neon](https://github.com/neon-bindings/neon)

## Building
1. Clone this repo
2. run `npm install` inside the root folder
3. Once everything is installed, run `neon build` to start building the native module (this will take some time initially)

## Basic Examples

### Playing a song
```js
const { Spotify } = require('../native');
const spotify = new Spotify('<username>', '<password>');

// Load specified track (by id) and starts playing
spotify.play('<track-id>');

setInterval(() => {
    console.log('playing? ', spotify.isPlaying());
}, 1000);
```

### Getting web token (can be used for retrieving metadata)
```js
const { Spotify } = require('../native');
const spotify = new Spotify('<username>', '<password>');

spotify.getToken('<spotify-client-id>', '<scopes>', (token) => {
    console.log(token.getToken(), token.getExpiry(), token.getScope());
});
```

## API (Work in progress)

```ts
interface Spotify {
    play(trackId: string);
    stop();
    pause();
    seek(positionMs: number);
    isPlaying(): boolean;
    getToken(): AccessToken;
}

interface AccessToken {
    getToken(): string;
    getExpiry(): number;
    getScope(): string[];
}
```

