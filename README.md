## Implementation 
* The wasm_bindgen code, just requires another module wasm_bindgen_futures compared to the previous implementation .

* Adding the deno.json and then running it using the commands below , generates the files in the lib folder , Now just declare the Log and Sleep functions in main.js , to test it . 

## Use
``` 
git clone git@github.com:Sahilgill24/runtime-poc.git
cd mod-rs
deno task wasmbuild
cd .. 
zinnia run main.js
```