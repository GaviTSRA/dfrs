"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    // If the extension is launched in debug mode then the debug server options are used
    // Otherwise the run options are used
    const serverOptions = {
        run: {
            command: "dfrs lsp",
            options: {
                shell: true
            }
        },
        debug: {
            command: "dfrs lsp",
            options: {
                shell: true
            }
        }
    };
    // Options to control the language client
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: '' }],
        // synchronize: {
        // 	// Notify the server about file changes to '.clientrc files contained in the workspace
        // 	fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
        // }
    };
    // Create the language client and start the client.
    client = new node_1.LanguageClient('dfrsLanguageServer', 'df.rs Language Server', serverOptions, clientOptions);
    // Start the client. This will also launch the server
    client.start();
}
exports.activate = activate;
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
exports.deactivate = deactivate;
