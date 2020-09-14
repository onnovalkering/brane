import { IRenderMime } from '@jupyterlab/rendermime-interfaces';
import { JSONObject } from '@lumino/coreutils';
import { Widget } from '@lumino/widgets';
import * as ReactDOM from 'react-dom';
import * as React from 'react';
import JSONTree from 'react-json-tree'
import { Tab, Tabs, TabList, TabPanel } from 'react-tabs';

const MIME_TYPE = 'application/vnd.brane.status+json';
const CLASS_NAME = 'brane-output-renderer';

/**
 * A widget for rendering Brane output.
 */
export class OutputWidget extends Widget implements IRenderMime.IRenderer {
  private _mimeType: string;

  constructor(options: IRenderMime.IRendererOptions) {
    super();

    this._mimeType = options.mimeType;
    this.addClass(CLASS_NAME);
  }

  /**
   * Render Brane status data into this widget's node.
   */
  renderModel(model: IRenderMime.IMimeModel): Promise<void> {
    return new Promise((resolve, reject) => {
      const data = model.data[this._mimeType] as JSONObject;
      const invocation: JSONObject = data["invocation"] as JSONObject;
      const instructions = JSON.parse(invocation["instructions_json"] as string);
      let return_json = invocation["return_json"] as string;

      let output = "";
      let outputClassName = "hide";
      if (return_json) {
        outputClassName = "output-renderer__output";

        const value = JSON.parse(return_json)
        console.log(value);

        const variant = value["v"];
        const content = value["c"];
        switch (variant) {
          case "unicode":
          case "integer":
          case "boolean":
          case "real":
            output = `${content}`
            break;
          default:
            output = JSON.stringify(content)
            break;
        }
      }

      // const spinnerRef = React.createRef<HTMLDivElement>();
      ReactDOM.render(
        <Tabs>
          <TabList>
            <Tab>Output</Tab>
            <Tab>Instructions</Tab>
          </TabList>
      
          <TabPanel>
            <div className={outputClassName}>
              {output}
            </div>
          </TabPanel>

          <TabPanel>
            <div className="output-renderer__instructions">
              <JSONTree
                  data={instructions}
                  hideRoot={true}
                  theme={"google"} 
                  getItemString={(type, data, itemType, itemString) => 
                    <span className="uppercase">{data["variant"]}</span>
                  }
              />
            </div>
          </TabPanel>
        </Tabs>,
        this.node, 
        () => { 
          resolve();
        }
      );
    });
  }
}

/**
 * A mime renderer factory for Brane output data.
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
      name: 'output_renderer',
      mimeTypes: [MIME_TYPE],
      extensions: []
    }
  ],
  documentWidgetFactoryOptions: {
    name: 'Output Renderer',
    primaryFileType: 'output-renderer',
    fileTypes: ['output-renderer'],
    defaultFor: ['output-renderer']
  }
};

export default extension;
