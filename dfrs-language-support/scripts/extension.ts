import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
	// If the extension is launched in debug mode then the debug server options are used
	// Otherwise the run options are used
	const serverOptions: ServerOptions = {
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
	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: '' }],
		// synchronize: {
		// 	// Notify the server about file changes to '.clientrc files contained in the workspace
		// 	fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		// }
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'dfrsLanguageServer',
		'df.rs Language Server',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
