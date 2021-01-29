import { JSONObject } from '@lumino/coreutils';
import * as ReactDOM from 'react-dom';
import * as React from 'react';
import JSONTree from 'react-json-tree'
import { Tab, Tabs, TabList, TabPanel } from 'react-tabs';
import BarLoader from "react-spinners/BarLoader";
import { DateTime, Settings } from "luxon";

type Props = {
    inProgress: boolean;
    instructions: any;
    output: string;
    info: {
        status: string,
        created: string,
        started: string,
        stopped: string,
    };
}

class Renderer {
    private _invocation: any;
    private _container: HTMLElement;

    constructor(invocation: JSONObject, container: HTMLElement, callback: Function) {
        this._invocation = invocation;
        this._container = container;

        this._render(callback);
    }

    /**
     *
     * @param invocation
     * @param callback
     */
    update(invocation: JSONObject, callback: Function) {
        console.log("ReactApp: update()");

        if (invocation.status != this._invocation.status) {
            this._invocation = invocation;
            this._render(callback);
        }
    }

    /**
     *
     * @param invocation
     */
    deriveProps(invocation: JSONObject): Props {
        const inProgress = invocation["status"] != "complete";
        const instructions = JSON.parse(invocation["instructions_json"] as string);
        const output = this.formatOutput(invocation["return_json"] as string);

        function formatDateTime(dt: string): string {
            if (!dt) {
                return "";
            }

            return DateTime.fromISO(dt, { zone: 'UTC' })
                .setZone(Settings.defaultZoneName)
                .toLocaleString(DateTime.DATETIME_FULL_WITH_SECONDS);
        }

        return {
            inProgress,
            instructions,
            output,
            info: {
                status: invocation["status"] as string,
                created: formatDateTime(invocation["created"] as string),
                started: formatDateTime(invocation["started"] as string),
                stopped: formatDateTime(invocation["stopped"] as string),
            },
        };
    }

    formatOutput(return_json: string): string {
        let output = "";
        if (return_json) {
            const value = JSON.parse(return_json)
            console.log(value);

            const variant = value["v"];
            const content = value["c"];
            switch (variant) {
                case "boolean":
                case "integer":
                case "real":
                case "unicode":
                    output = `${content}`
                    break;
                case "struct":
                    const type = content["type"];
                    switch (type) {
                        case "Directory":
                        case "File":
                            output = content["properties"]["url"]["c"]
                            break;
                        default:
                            output = JSON.stringify(content)
                            break;
                    }
                    break;
                default:
                    output = JSON.stringify(content)
                    break;
            }
        }

        return output;
    }

    /**
     *
     * @param callback
     */
    _render(callback: Function) {
        console.log("ReactApp: _render()");
        const { inProgress, instructions, output, info } = this.deriveProps(this._invocation);

        ReactDOM.render(
            <App inProgress={inProgress} instructions={instructions} output={output} info={info} />,
            this._container,
            () => callback(),
        );
    }
}

class App extends React.Component<Props> {
    /**
     *
     */
    render() {
        console.log("App: render()");

        return (
            <Tabs>
                <TabList>
                    <Tab>Output</Tab>
                    <Tab>Information</Tab>
                    <Tab>IR</Tab>
                </TabList>

                <TabPanel>
                    <div className="invocation-renderer__output">
                        {this.props.inProgress
                          ? <BarLoader color="#000" css="display: block" width={54} height={4} />
                          : this.props.output
                        }
                    </div>
                </TabPanel>

                <TabPanel>
                    <div className="invocation-renderer__information">
                        <table>
                            <tr>
                                <td>Status:</td>
                                <td><span className="uppercase">{this.props.info.status}</span></td>
                            </tr>
                            <tr>
                                <td>Created:</td>
                                <td>{this.props.info.created}</td>
                            </tr>
                            <tr>
                                <td>Started:</td>
                                <td>{this.props.info.started}</td>
                            </tr>
                            <tr>
                                <td>Stopped:</td>
                                <td>{this.props.info.stopped}</td>
                            </tr>
                        </table>
                    </div>
                </TabPanel>

                <TabPanel>
                    <div className="invocation-renderer__ir">
                        <JSONTree
                            data={this.props.instructions}
                            hideRoot={true}
                            theme={"google"}
                            getItemString={(type, data, itemType, itemString) =>
                                <span className="uppercase">{data["variant"]}</span>
                            }
                        />
                    </div>
                </TabPanel>
            </Tabs>
        );
    }
}

export default Renderer;
