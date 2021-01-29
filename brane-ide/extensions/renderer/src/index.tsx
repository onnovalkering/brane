import { IRenderMime } from '@jupyterlab/rendermime-interfaces';
import { JSONObject } from '@lumino/coreutils';
import { Widget } from '@lumino/widgets';
import Renderer from './component';

const MIME_TYPE = 'application/vnd.brane.invocation+json';
const CLASS_NAME = 'brane-invocation-renderer';

/**
 * A widget for rendering the status of a Brane invocation.
 */
export class RendererWidget extends Widget implements IRenderMime.IRenderer {
    private _mimeType: string;
    private _renderer: Renderer | null;

    constructor(options: IRenderMime.IRendererOptions) {
        super();

        this._mimeType = options.mimeType;
        this._renderer = null;
        this.addClass(CLASS_NAME);
    }

    renderModel(model: IRenderMime.IMimeModel): Promise<void> {
        const data = model.data[this._mimeType] as JSONObject;
        const invocation = data["invocation"] as JSONObject;

        return new Promise((resolve) => {
            if (!this._renderer) {
                this._renderer = new Renderer(invocation, this.node, resolve);
            } else {
                this._renderer.update(invocation, resolve);
            }
        });
    }
}

/**
 * Extension definition.
 */
const extension: IRenderMime.IExtension = {
    id: 'invocation_renderer:plugin',
    rendererFactory: {
        safe: true,
        mimeTypes: [MIME_TYPE],
        createRenderer: options => new RendererWidget(options)
    },
    rank: 0,
    dataType: 'json',
    fileTypes: [
        {
            name: 'invocation_renderer',
            mimeTypes: [MIME_TYPE],
            extensions: []
        }
    ],
    documentWidgetFactoryOptions: {
        name: 'Invocation Renderer',
        primaryFileType: 'invocation-renderer',
        fileTypes: ['invocation-renderer'],
        defaultFor: ['invocation-renderer']
    }
};

export default extension;
