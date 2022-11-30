"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.map_replacer = exports.EscrowAuthority = exports.loadSolanaConfigFile = exports.use_metaplex = void 0;
const os_1 = __importDefault(require("os"));
const yaml_1 = __importDefault(require("yaml"));
const js_1 = require("@metaplex-foundation/js");
const web3 = __importStar(require("@solana/web3.js"));
const fs = __importStar(require("fs"));
async function use_metaplex(keypair, env, rpc) {
    const solanaConfig = (0, exports.loadSolanaConfigFile)();
    let connection;
    const selectedRPC = rpc || solanaConfig.json_rpc_url;
    const selectedKeypairPath = keypair || solanaConfig.keypair_path;
    if (selectedRPC) {
        connection = new web3.Connection(selectedRPC, {
            confirmTransactionInitialTimeout: 360000,
        });
    }
    else {
        connection = new web3.Connection(web3.clusterApiUrl(env), {
            confirmTransactionInitialTimeout: 360000,
        });
    }
    // Load a local keypair.
    const keypairFile = fs.readFileSync(selectedKeypairPath);
    const wallet = web3.Keypair.fromSecretKey(Buffer.from(JSON.parse(keypairFile.toString())));
    const metaplex = new js_1.Metaplex(connection);
    // Use it in the SDK.
    metaplex.use((0, js_1.keypairIdentity)(wallet));
    return metaplex;
}
exports.use_metaplex = use_metaplex;
const loadSolanaConfigFile = () => {
    try {
        const path = os_1.default.homedir() + '/.config/solana/cli/config.yml';
        const solanaConfigFile = fs.readFileSync(path);
        const config = yaml_1.default.parse(solanaConfigFile.toString());
        return config;
    }
    catch (e) {
        return {};
    }
};
exports.loadSolanaConfigFile = loadSolanaConfigFile;
var EscrowAuthority;
(function (EscrowAuthority) {
    EscrowAuthority[EscrowAuthority["TokenOwner"] = 0] = "TokenOwner";
    EscrowAuthority[EscrowAuthority["Creator"] = 1] = "Creator";
})(EscrowAuthority = exports.EscrowAuthority || (exports.EscrowAuthority = {}));
// Creating a replacer to properly JSON stringify Maps.
function map_replacer(key, value) {
    if (value instanceof Map) {
        return {
            dataType: 'Map',
            value: Array.from(value.entries()), // or with spread: value: [...value]
        };
    }
    else if (value instanceof Set) {
        return {
            dataType: 'Set',
            value: Array.from(value.values()),
        };
    }
    else {
        return value;
    }
}
exports.map_replacer = map_replacer;
