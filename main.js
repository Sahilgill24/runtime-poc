import { run } from './mod-rs/lib/mod_rs.js'

// These tell the rust code that the zinnia_log was this function
// in actual
Zinnia.log = (msg) => { console.log(msg); };
Zinnia.sleep = (ms) => {
    return new Promise(r => setTimeout(r, Number(ms)));
};
// 
// const Zinnia = {
//     log(msg) {
//         Deno.core.op_log(msg);
// here would have to add Deno.core.ops etc. 
//     },
//     sleep(ms) {
//         return Deno.core.ops.op_sleep(ms);
//     }
// }

// Zinnia.log("99");
// await Zinnia.sleep(1000);
// Zinnia.log("1000")

(async () => {
    await run();
})();