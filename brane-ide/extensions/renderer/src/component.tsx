import { JSONObject } from '@lumino/coreutils';
import * as ReactDOM from 'react-dom';
import * as React from 'react';
import { Tab, Tabs, TabList, TabPanel } from 'react-tabs';
import BarLoader from "react-spinners/BarLoader";

type Props = {
    inProgress: boolean;
    output: string;
    bytecode: string;
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
        const inProgress = !(invocation["done"] as boolean);
        const output = invocation["output"] as string;
        const bytecode = invocation["bytecode"] as string;

        return {
            inProgress,
            output,
            bytecode,
        };
    }

    /**
     *
     * @param callback
     */
    _render(callback: Function) {
        console.log("ReactApp: _render()");
        const { inProgress, output, bytecode } = this.deriveProps(this._invocation);

        ReactDOM.render(
            <App inProgress={inProgress} output={output} bytecode={bytecode} />,
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
        const bytecode_opcodes = this.props.bytecode.split("\n").filter(x => x.length > 0);
        const bytecode_list_items = [];

        for (let opcode of bytecode_opcodes) {
            bytecode_list_items.push(<li key={opcode}>{opcode}</li>);
        }

        return (
            <Tabs>
                <TabList>
                    <Tab>Output</Tab>
                    <Tab>Bytecode</Tab>
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
                    <div className="invocation-renderer__bytecode">
                        <ul>{bytecode_list_items}</ul>
                    </div>
                </TabPanel>
            </Tabs>
        );
    }
}

export default Renderer;
