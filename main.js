import { run } from './mod-rs/lib/mod_rs.js'

Zinnia.log = (msg) => { console.log(msg); };
Zinnia.sleep = (ms) => {
    return new Promise(r => setTimeout(r, Number(ms)));
};
// This is a Javascript API only 
// const Zinnia = {
//     log(msg) {
//         console.log(msg);
// here would have to add Deno.core.ops etc. 
//     },
//     sleep(ms) {
//         return new Promise(resolve => setTimeout(resolve, ms));
//     }
// }

// Zinnia.log("99");
// await Zinnia.sleep(1000);
// Zinnia.log("1000")

(async () => {
    await run();
})();