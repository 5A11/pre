import React from "react";
import { AxiosResponse } from "axios";

import { getApi } from "../apis/Requests";
import ResponseDataAccess from "../intefaces/ResponseDataAccess";

import DataAccessEntry from "./DataAccessEntry";
import AddDataForm from "./AddDataForm";
import DeleteDataAccessBtn from "./DeleteDataAccessBtn";
import DownloadDataBtn from "./DownloadDataBtn";
import ManageAccessForm from "./ManageAccessForm";

import style from "./DataAccessTable.module.scss"
import dataAccessEntryStyle from "./DataAccessEntry.module.scss"


interface DataAccessOwnedState {
    dataAccesses: Array<ResponseDataAccess>
    selectedDataAccess: ResponseDataAccess | null
}

interface DataAccessOwnedProps {
}

class DataAccessOwned extends React.Component<DataAccessOwnedProps, DataAccessOwnedState> {

    constructor(props: DataAccessOwnedProps) {
        super(props);

        this.state = {
            dataAccesses: [],
            selectedDataAccess: null
        };
    }

    async apiGetDataAccesses(): Promise<void> {
        const url = "/data-accesses/owned";
        const result = (await getApi(url)) as AxiosResponse;
        if (result.status === 200) {
            this.setState({dataAccesses: result.data})
        } 
        else {
            // TODO: add better error handling
            alert(`Failed to get owned Data accesses. ${JSON.stringify(result.data)}`);
        }
    }

    async componentDidMount(): Promise<void> {
        await this.apiGetDataAccesses()
        
    }

    handleDataAccessCheckboxChange(event: React.ChangeEvent<HTMLInputElement>): void {
        const dataAccess = this.state.dataAccesses.find(dataAccess => {
            return dataAccess.id === Number(event.target.value)
        })
        if (this.state.selectedDataAccess !== null || dataAccess === undefined) {
            this.setState({ selectedDataAccess:  null});
        }
        else {
            this.setState({ selectedDataAccess:  dataAccess});
        }
    }

    isCheckboxDisabled(dataAccessId: number): boolean {
        return (
            this.state.selectedDataAccess !== null
            && dataAccessId !== this.state.selectedDataAccess.id
        )
    }

    getDataAccessTable(): Array<JSX.Element> {

        const entries: Array<JSX.Element> = [
            <div key="header">
                <div className={dataAccessEntryStyle.Row}>
                    <div className={dataAccessEntryStyle.CheckboxCol}><input type="checkbox" disabled /></div>
                    <div className={dataAccessEntryStyle.Col}>Data ID</div>
                    <div className={dataAccessEntryStyle.Col}>Readers</div>
                </div>
                <hr />
            </div>
        ]
        for (const dataAccess of this.state.dataAccesses) {
            entries.push(
                <DataAccessEntry 
                    dataAccess={dataAccess} 
                    isOwned={true}
                    handleDataAccessCheckboxChange={this.handleDataAccessCheckboxChange.bind(this)} 
                    isCheckboxDisabled={this.isCheckboxDisabled(dataAccess.id)}
                    key={dataAccess.id}
                />
            )
        }
        return entries
    }

    render(): JSX.Element {
        return (
            <div>
                <h1>My data</h1>

                <AddDataForm />
                <DeleteDataAccessBtn dataAccess={this.state.selectedDataAccess} />
                <DownloadDataBtn dataAccess={this.state.selectedDataAccess} />
                <ManageAccessForm dataAccess={this.state.selectedDataAccess} />

                <button disabled title="Not implemented">Select Proxies</button>

                <div className={style.DataTable}>
                    {this.getDataAccessTable()}
                </div>
                
            </div>
        );
    }
}

export default DataAccessOwned;
