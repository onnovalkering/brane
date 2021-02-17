import {
    ILayoutRestorer, JupyterFrontEnd, JupyterFrontEndPlugin
} from '@jupyterlab/application';

import {
    ICommandPalette, MainAreaWidget, WidgetTracker
} from '@jupyterlab/apputils';

import {
    ISettingRegistry
} from '@jupyterlab/settingregistry';

import {
    ILauncher
} from '@jupyterlab/launcher';

import {
    Message
} from '@lumino/messaging';

import {
    Widget
} from '@lumino/widgets';

import * as ReactDOM from 'react-dom';
import * as React from 'react';

const PLUGIN_ID = 'brane-vault:vault';
const VAULT_ICON_CLASS = "b-VaultIcon"

interface Secrets {
    keys: String[]
}

class VaultWidget extends Widget {
    private apiHost: String;
    private secrets: Secrets | null;

    constructor(apiHost: String) {
        super();

        this.addClass('brane-vault');
        this.apiHost = apiHost;
        this.secrets = null;
    }

    /**
     * Handle update requests for the widget.
     */
    async onUpdateRequest(msg: Message): Promise<void> {
        if (!this.secrets) {
            this.secrets = await this.getSecrets(this.apiHost);
        }

        ReactDOM.render(
            <div>
                <h1>Vault</h1>
            </div>,
            this.node
        );
    }

    /**
     * Retreives a list of packages from the Brane API.
     */
    private async getSecrets(apiHost: String): Promise<Secrets> {
        const response = await fetch(`http://${apiHost}/vault`);
        if (!response.ok) {
            alert(`Failed to reach Brane API at ${apiHost}!`);
            return { keys: [] };
        }

        return await response.json() as Secrets;
    }
}

/**
 * Activate the Vault widget extension.
 */
async function activate(app: JupyterFrontEnd, palette: ICommandPalette, launcher: ILauncher, restorer: ILayoutRestorer, settings: ISettingRegistry) {
    console.log('JupyterLab extension "vault" is activated!');

    const registrySettings = await settings.load(PLUGIN_ID);

    // Declare a widget variable
    let widget: MainAreaWidget<VaultWidget>;

    // Add an application command
    const command: string = 'brane:open-vault';
    app.commands.addCommand(command, {
        label: 'Vault',
        iconClass: args => (args['isPalette'] ? '' : VAULT_ICON_CLASS),
        execute: () => {
            if (!widget || widget.isDisposed) {
                const apiHost = registrySettings.get("apiHost").composite as string;

                // Create a new widget if one does not exist
                // or if the previous one was disposed after closing the panel
                const content = new VaultWidget(apiHost);
                widget = new MainAreaWidget({ content });
                widget.id = 'vault';
                widget.title.label = 'vault';
                widget.title.closable = true;
                widget.title.iconClass = VAULT_ICON_CLASS;
            }
            if (!tracker.has(widget)) {
                // Track the state of the widget for later restoration
                tracker.add(widget);
            }
            if (!widget.isAttached) {
                // Attach the widget to the main work area if it's not there
                app.shell.add(widget, 'main');
            }

            widget.content.update();

            // Activate the widget
            app.shell.activateById(widget.id);
        }
    });

    // Add the command to the launcher
    if (launcher) {
        launcher.add({
            command,
            category: 'Brane',
            rank: 1,
        });
    }

    // Add the command to the palette.
    palette.addItem({ command, category: 'Brane' });

    // Track and restore the widget state
    let tracker = new WidgetTracker<MainAreaWidget<VaultWidget>>({
        namespace: 'brane'
    });
    restorer.restore(tracker, {
        command,
        name: () => 'brane'
    });
}


/**
 * Initialization data for the Vault extension.
 */
const extension: JupyterFrontEndPlugin<void> = {
    id: PLUGIN_ID,
    autoStart: true,
    requires: [ICommandPalette, ILauncher, ILayoutRestorer, ISettingRegistry],
    activate: activate
};

export default extension;
