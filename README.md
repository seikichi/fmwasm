# fmwasm

[FM-index](https://en.wikipedia.org/wiki/FM-index) implementation written in Rust.
This library also generates WebAssembly library so that you can use this **full-text search engine in browsers**.

I got an idea from [elasticlunr](http://elasticlunr.com/) about client-side full-text search engine.
Compared to [elasticlunr](http://elasticlunr.com/), this library has following advantages:

- Small index file (TODO: do experiments)
- No need of tokenizer (useful for CJK languages)

## NPM Library

T.B.D.

## Example

NOTE: You can find more examples in `examples` directory.

### Create Index File

```sh
> echo "京都府京都市左京区吉田本町" | cargo run --release --example constract > index.bincode
```

### Load Index File from Browser

```js
(async () => {
  const response = await fetch("/index.bincode");
  const buffer = await response.arrayBuffer();

  const { FMIndex } = await import("./node_modules/@seikichi/fmwasm/fmwasm.js");
  const fmindex = FMIndex.from(new Uint8Array(buffer));
  console.log(fmindex.counts("京都")); // -> 2

  const query = "京";
  const { start, end } = fmindex.search(query);
  for (let i = start; i < end; i++) {
    console.log(`${fmindex.previous_string(i, 3)}${query}`);
    // -> 都市左京
    // -> 京都府京
    // -> 京
  }
})();
```

## References

- [herumi/fmindex](https://github.com/herumi/fmindex)
- [rust-bio/rust-bio](https://github.com/rust-bio/rust-bio)
- [sekineh/wavelet-matrix-rs](https://github.com/sekineh/wavelet-matrix-rs)
- [MitI-7/WaveletMatrix](https://github.com/MitI-7/WaveletMatrix)

