import { IRenderMime } from '@jupyterlab/rendermime-interfaces';


import { JSONObject } from '@lumino/coreutils';


import { Widget } from '@lumino/widgets';

/**
 * The default mime type for the extension.
 */
const MIME_TYPE = 'application/vnd.brane.status+json';

/**
 * The class name added to the extension.
 */
const CLASS_NAME = 'brane-status-renderer';

/**
 * A widget for rendering Brane statuses.
 */
export class OutputWidget extends Widget implements IRenderMime.IRenderer {
  /**
   * Construct a new output widget.
   */
  constructor(options: IRenderMime.IRendererOptions) {
    super();
    this._mimeType = options.mimeType;
    this.addClass(CLASS_NAME);
  }

  /**
   * Render Brane status data into this widget's node.
   */
  renderModel(model: IRenderMime.IMimeModel): Promise<void> {
    let data = model.data[this._mimeType] as JSONObject;
    this.node.textContent = JSON.stringify(data);
    
    return Promise.resolve();
  }

  private _mimeType: string;
}

/**
 * A mime renderer factory for Brane status data.
 */
export const rendererFactory: IRenderMime.IRendererFactory = {
  safe: true,
  mimeTypes: [MIME_TYPE],
  createRenderer: options => new OutputWidget(options)
};

/**
 * Extension definition.
 */
const extension: IRenderMime.IExtension = {
  id: 'status_renderer:plugin',
  rendererFactory,
  rank: 0,
  dataType: 'json',
  fileTypes: [
    {
      name: 'status_renderer',
      mimeTypes: [MIME_TYPE],
      extensions: []
    }
  ],
  documentWidgetFactoryOptions: {
    name: 'Status Renderer',
    primaryFileType: 'status-renderer',
    fileTypes: ['status-renderer'],
    defaultFor: ['status-renderer']
  }
};

export default extension;
