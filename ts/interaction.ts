import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import fs from "fs";

// Define the sender's private key
// const privateKey = "your_private_key_here";

// Create a signer object using the private key
// const wallet = await DirectSecp256k1Wallet.fromKey(privateKey);
// const mnemonic = 

// Create a wallet from the mnemonic
const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: "neutron",
});

// Initialize a CosmWasm client with the signer
const client = await SigningCosmWasmClient.connectWithSigner("https://rpc-palvus.pion-1.ntrn.tech", wallet, {
    gasPrice: GasPrice.fromString("0.025untrn"),
});

// Define the sender's address and the contract address
const [account] = await wallet.getAccounts();
const senderAddress = account.address;
console.log(senderAddress)

// deploy

const wasm= fs.readFileSync("/Users/macbookpro/Downloads/excalidraw/nft_staking/artifacts/nft_staking.wasm")
const result = await client.upload(senderAddress, wasm, "auto")
console.log(result)

// instantiate

const codeId = result.codeId; // 
const nftCodeId = 3471
const nftInstantiateMsg = {"minter": senderAddress, "name": "Token", "symbol": "TOKEN"}
const nftInstantiateResponse = await client.instantiate(senderAddress, nftCodeId, nftInstantiateMsg, "NFT", "auto")
const nftcontractAddress = nftInstantiateResponse.contractAddress

//const nftcontractAddress = "neutron18lm98glhpyx6z44f9vzj3gs697h7l9n0rdmg4zyvtvkmv7vhjrkskz29ft"

//Define the instantiate message
const instantiateMsg = { "admin": senderAddress, "nft_addr": nftcontractAddress }; // for the staking contract

//Instantiate the contract
const instantiateResponse = await client.instantiate(senderAddress, codeId, instantiateMsg, "NFT Staking", "auto")
console.log(instantiateResponse)

const contractAddress = instantiateResponse.contractAddress

// Define the token ID and owner
const tokenId = "send_try"; 
const owner = senderAddress; // set the owner to the sender's address

const metadata = {
    name: "Token Name",
    description: "Token Description",
};

// mint NFT first
const mintResult = await client.execute(senderAddress, nftcontractAddress, {mint: {token_id: tokenId, extension: metadata, owner: senderAddress}}, "auto")

// send function directly triggers staking
const sendNft = await client.execute(senderAddress, nftcontractAddress, {send_nft: {contract: contractAddress, msg: "" , token_id: "send_try"}}, "auto" );

// index should start from 0
const unstakeResult = await client.execute(senderAddress, contractAddress, {unstake: {index: 0}}, "auto")

// claim 
const claimResult = await client.execute(senderAddress, contractAddress, {claim: {index: 0}}, "auto")

// check if it is transferred back
const TokensResponse = await client.queryContractSmart(nftcontractAddress, { tokens: { owner: senderAddress }});
console.log(TokensResponse)

// admin can add nft addresses to whitelist
const queryCollection = await client.queryContractSmart(contractAddress, {whitelisted_nft_addresses: {}}) // before
const nftcontractAddress2 = "neutron1e7yppujrshzzsqfflu09udrje0zpd6jnfwe304wdexl7dd28gqxqv8x776";
const addCollectionResult = await client.execute(senderAddress, contractAddress, {add_collection: {nft_addr: nftcontractAddress2}}, "auto")

// query collection
const queryCollection2 = await client.queryContractSmart(contractAddress, {whitelisted_nft_addresses: {}}) // after

const removeCollectionResult = await client.execute(senderAddress, contractAddress, {remove_collection: {nft_addr: nftcontractAddress2}}, "auto")
// check if it is removed
const queryCollection3 = await client.queryContractSmart(contractAddress, {whitelisted_nft_addresses: {}}) 

// query stakings
const queryStaking = await client.queryContractSmart(contractAddress, {stakings_by_address: {address: senderAddress}})

// query admin
const queryAdmin = await client.queryContractSmart(contractAddress, {admin_address: {}})

// Admin can burn the NFT
const adminBurnResult = await client.execute(senderAddress, contractAddress, {admin_burn: {index: 0}}, "auto")