import { IRenderMime } from '@jupyterlab/rendermime-interfaces';
import { toArray } from '@lumino/algorithm';
import { Widget } from '@lumino/widgets';

/**
 * The default mime type for the extension.
 */
const MIME_TYPE = 'image/fits';

/**
 * The class name added to the extension.
 */
const CLASS_NAME = 'mimerenderer-js9';

/**
 * A widget that wraps JS9.
 */
export class Js9Widget extends Widget implements IRenderMime.IRenderer {
  private _mimeType: string;
  private _uniqueId: string;

  /**
   * Construct a new JS9 widget.
   */
  constructor(options: IRenderMime.IRendererOptions) {
    super();
    this._mimeType = options.mimeType;
    this._uniqueId = 'js9_' + Math.random().toString().substr(2, 6);
    this.addClass(CLASS_NAME);
  }

  /**
   * Render the JS9 widget in the target HTML node.
   */
  async renderModel(model: IRenderMime.IMimeModel): Promise<void> {
    if (!window.hasOwnProperty("JS9")) {
      await loadJs9(this.node);
    }

    return new Promise<void>(resolve => {
      const w = window as any;
      let image = model.data[this._mimeType] as string

      // Add widget's HTML elements.
      const display = this.renderHtmlElements();

      // Initialize JS9 display
      new w.JS9.Display(display);
      w.JS9.Load(image, {
        onload: () => {
          w.JS9.instantiatePlugins();
          resolve();
        }
      }, { "display": this._uniqueId });
    });
  }

  /**
   * Add the individual widget's elements to the DOM.
   */
  renderHtmlElements() {
    // Add menubar-div element
    const menubar = document.createElement('div');
    menubar.className = "JS9Menubar";
    menubar.id = this._uniqueId + "Menubar";
    menubar.setAttribute('data-displays', this._uniqueId);
    this.node.appendChild(menubar);

    // Add display-div element
    const display = document.createElement('div');
    display.className = "JS9";
    display.id = this._uniqueId;
    this.node.appendChild(display);

    // Add margin-div element
    const margin = document.createElement('div');
    margin.setAttribute('style', 'margin-top: 2px;');
    this.node.appendChild(margin);

    // Add colorbar-div element
    const colorbar = document.createElement('div');
    colorbar.className = "JS9Colorbar";
    colorbar.id = this._uniqueId + "Colorbar";
    this.node.appendChild(colorbar);

    return display;
  }
}

/**
 * Inject the JS9 all-in-one script and stylesheet.
 * This method polls until JS9 has been initialized.
 */
async function loadJs9(el: HTMLElement): Promise<void> {
  const w = window as any;

  if (!w.js9Inserted) {
    w.js9Inserted = true;
    console.log("JS9 all-in-one script and stylesheet inserted.");

    el.insertAdjacentHTML('beforeend', `
      <link type="text/css" rel="stylesheet" href="http://js9.si.edu/js9/js9-allinone.css" />
      <script type="text/javascript" src="http://js9.si.edu/js9/js9-allinone.js"></script>
    `);

    evalInnerHTMLScriptTags(el);
  }

  const poll = (resolve: any) => {
    setTimeout(() => {
      if (window.hasOwnProperty("JS9") && w.JS9.inited) {
        w.JS9.mousetouchZoom = true;
        resolve();
      } else {
        poll(resolve);
      }
    }, 1000);
  };

  // Wait until JS9 has been initialized.
  await new Promise<void>(poll);
}

/**
 * Eval the script tags contained in a node populated by `innerHTML`.
 *
 * When script tags are created via `innerHTML`, the browser does not
 * evaluate them when they are added to the page. This function works
 * around that by creating new equivalent script nodes manually, and
 * replacing the originals.
 *
 * @copyright Jupyter Development Team
 * @from https://github.com/jupyterlab/jupyterlab
 * @license BSD
 */
function evalInnerHTMLScriptTags(el: HTMLElement): void {
  // Create a snapshot of the current script nodes.
  const scripts = toArray(el.getElementsByTagName('script'));

  // Loop over each script node.
  for (const script of scripts) {
    // Skip any scripts which no longer have a parent.
    if (!script.parentNode) {
      continue;
    }

    // Create a new script node which will be clone.
    const clone = document.createElement('script');
    clone.async = true;

    // Copy the attributes into the clone.
    const attrs = script.attributes;
    for (let i = 0, n = attrs.length; i < n; ++i) {
      const { name, value } = attrs[i];
      clone.setAttribute(name, value);
    }

    // Copy the text content into the clone.
    clone.textContent = script.textContent;

    // Replace the old script in the parent.
    script.parentNode.replaceChild(clone, script);
  }
}

/**
 * A mime renderer factory for JS9.
 */
export const rendererFactory: IRenderMime.IRendererFactory = {
  safe: true,
  mimeTypes: [MIME_TYPE],
  createRenderer: options => new Js9Widget(options)
};

/**
 * Extension definition.
 */
const extension: IRenderMime.IExtension = {
  id: 'brane-js9:plugin',
  rendererFactory,
  rank: 0,
  dataType: 'string',
  fileTypes: [
    {
      name: 'fits',
      mimeTypes: [MIME_TYPE],
      extensions: ['.fits']
    }
  ],
  documentWidgetFactoryOptions: {
    name: 'JS9',
    primaryFileType: 'fits',
    fileTypes: ['fits'],
    defaultFor: ['fits']
  }
};

export default extension;
