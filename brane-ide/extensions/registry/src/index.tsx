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

  import {
    Accordion,
    AccordionItem,
    AccordionItemHeading,
    AccordionItemButton,
    AccordionItemPanel,
  } from 'react-accessible-accordion';

  import * as ReactDOM from 'react-dom';
  import * as React from 'react';

  const PLUGIN_ID = 'brane-registry:registry';
  const REGISTRY_ICON_CLASS = "b-RegistryIcon"

  interface Package {
    name: string,
    version: string,
    description?: string,
    functions?: Map<string, Function>,
  }

  interface Function {
    parameters: Parameter[],
    pattern: CallPattern,
    returnType: string,
  }

  interface Parameter {
    type: string,
    name: string,
    secret?: string,
  }

  interface CallPattern {
    infix?: string[],
    postfix?: string,
    prefix?: string
  }

  function RenderPackages(props: {packages: Package[]}) {
    return (
        <Accordion allowZeroExpanded={true} allowMultipleExpanded={true}>
          {props.packages.map(p => (
            <AccordionItem key={p.name}>
              <AccordionItemHeading>
                  <AccordionItemButton>
                      {p.name}
                  </AccordionItemButton>
              </AccordionItemHeading>
              <AccordionItemPanel>
                  <p>{p.description || "No description available."}</p>
                  <RenderFunctions functions={p.functions} />
              </AccordionItemPanel>
          </AccordionItem>
          ))}
        </Accordion>
    );
  }

  function RenderFunctions(props: {functions?: Map<string, Function>}) {
    if (!props.functions) {
      return <p>This package doesn't contain any functions.</p>
    }

    let functions: any = props.functions;

    // For some reason .keys() / .entries() doesn't work in the browser... (?)
    let names = [];
    for (let name in functions) {
      names.push(name);
    }

    return (
      <div className="functions">
        {names.map(name => RenderFunction(name, functions[name]))}
      </div>
    )
  }

  function RenderFunction(name: string, f: Function) {
    return (
      <div className="function" key={name}>
        <p><strong>{name}</strong>: {f.returnType}</p>
        <div className="pattern">
          <RenderPattern name={name} parameters={f.parameters} pattern={f.pattern} />
        </div>
      </div>
    )
  }

  function RenderPattern(props: {name: string, parameters: Parameter[], pattern: CallPattern}) {
    const name = props.name;
    const parameters = props.parameters;
    const pattern = props.pattern;

    if (!pattern) {
      let sb = name;

      parameters.forEach(p => {
        sb += ` <${p.type}>`
      });

      return <p>{sb}</p>;
    }

    let infixes = pattern.infix || [];
    return (
      <p>
        <strong>{`${pattern.prefix || ''} `}</strong>

        {parameters.filter(p => !p.secret).map((p, i) => (
          <span>
            <span>{`<${p.type}> `}</span>
            <strong>{`${infixes[i] || ''} `}</strong>
          </span>
        ))}

        <strong>{`${pattern.postfix || ''} `}</strong>
      </p>
    )
  }

  class RegistryWidget extends Widget {
    private apiHost: String;
    private packages: Package[] | null;

    constructor(apiHost: String) {
      super();

      this.addClass('brane-registry');
      this.apiHost = apiHost;
      this.packages = null;
    }

    // /**
    //  * The summary text element associated with the widget.
    //  */
    // readonly summary: HTMLParagraphElement;

    /**
     * Handle update requests for the widget.
     */
    async onUpdateRequest(msg: Message): Promise<void> {
      if (!this.packages) {
        this.packages = await this.getPackages(this.apiHost);
      }

      ReactDOM.render(
        <div>
          <h1>Registry</h1>
          <RenderPackages packages={this.packages} />
        </div>,
        this.node
      );
    }

    /**
     * Retreives a list of packages from the Brane API.
     */
    private async getPackages(apiHost: String): Promise<Package[]> {
      const response = await fetch(`http://${apiHost}/packages`);
      if (!response.ok) {
        alert(`Failed to reach Brane API at ${apiHost}!`);
        return [];
      }

      return await response.json() as Package[];
    }
  }

  /**
   * Activate the Registry widget extension.
   */
  async function activate(app: JupyterFrontEnd, palette: ICommandPalette, launcher: ILauncher, restorer: ILayoutRestorer, settings: ISettingRegistry) {
    console.log('JupyterLab extension "registry" is activated!');

    const registrySettings = await settings.load(PLUGIN_ID);

    // Declare a widget variable
    let widget: MainAreaWidget<RegistryWidget>;

    // Add an application command
    const command: string = 'brane:open-registry';
    app.commands.addCommand(command, {
      label: 'Registry',
      iconClass: args => (args['isPalette'] ? '' : REGISTRY_ICON_CLASS),
      execute: () => {
        if (!widget || widget.isDisposed) {
          const apiHost = registrySettings.get("apiHost").composite as string;

          // Create a new widget if one does not exist
          // or if the previous one was disposed after closing the panel
          const content = new RegistryWidget(apiHost);
          widget = new MainAreaWidget({content});
          widget.id = 'registry';
          widget.title.label = 'Registry';
          widget.title.closable = true;
          widget.title.iconClass = REGISTRY_ICON_CLASS;
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
