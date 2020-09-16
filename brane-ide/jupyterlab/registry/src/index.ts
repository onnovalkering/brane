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

const PLUGIN_ID = 'registry:registry';

class RegistryWidget extends Widget {
  constructor() {
    super();

    this.addClass('brane-registry');

    // Add a summary element to the panel
    this.summary = document.createElement('p');
    this.node.appendChild(this.summary);
  }

  /**
   * The summary text element associated with the widget.
   */
  readonly summary: HTMLParagraphElement;

  /**
   * Handle update requests for the widget.
   */
  async onUpdateRequest(msg: Message): Promise<void> {
    this.summary.innerText = "Hello, world!";
  }
}

/**
 * Activate the Registry widget extension.
 */
async function activate(app: JupyterFrontEnd, palette: ICommandPalette, launcher: ILauncher, restorer: ILayoutRestorer, settings: ISettingRegistry) {
  console.log('JupyterLab extension "registry" is activated!');

  // Declare a widget variable
  let widget: MainAreaWidget<RegistryWidget>;

  const s = await settings.load(PLUGIN_ID);
  const apiHost = s.get("apiHost").composite as string;

  alert(apiHost);

  // Add an application command
  const command: string = 'brane:open-registry';
  app.commands.addCommand(command, {
    label: 'Registry',
    execute: () => {
      if (!widget || widget.isDisposed) {
        // Create a new widget if one does not exist
        // or if the previous one was disposed after closing the panel
        const content = new RegistryWidget();
        widget = new MainAreaWidget({content});
        widget.id = 'registry';
        widget.title.label = 'Registry';
        widget.title.closable = true;
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
      rank: 1
    });
  }  

  // Add the command to the palette.
  palette.addItem({ command, category: 'Brane' });

  // Track and restore the widget state
  let tracker = new WidgetTracker<MainAreaWidget<RegistryWidget>>({
    namespace: 'brane'
  });
  restorer.restore(tracker, {
    command,
    name: () => 'brane'
  });
}


/**
 * Initialization data for the Registry extension.
 */
const extension: JupyterFrontEndPlugin<void> = {
  id: PLUGIN_ID,
  autoStart: true,
  requires: [ICommandPalette, ILauncher, ILayoutRestorer, ISettingRegistry],
  activate: activate
};

export default extension;