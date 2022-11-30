"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const msgpack_1 = require("@msgpack/msgpack");
const encoded = (0, msgpack_1.encode)({ foo: 'bar' });
console.log(encoded);
const decoded = (0, msgpack_1.decode)(encoded);
console.log(decoded);
