import React from "react";
import { AxiosResponse } from "axios";

import { deletetApi } from "../apis/Requests";
import ResponseDataAccess from "../intefaces/ResponseDataAccess";


interface DeleteDataAccessBtnState { }

interface DeleteDataAccessBtnProps {
    dataAccess: ResponseDataAccess | null
    // setParentState: void
}

class DeleteDataAccessBtn extends React.Component<DeleteDataAccessBtnProps, DeleteDataAccessBtnState> {

    async apiDeleteData(): Promise<void> {
        if (!this.props.dataAccess) {
            return
        }
        const url = `/data-accesses/${this.props.dataAccess.id}`;
        const result = (await deletetApi(url)) as AxiosResponse;
        if (result.status === 204) {
            alert("Data is successfully deleted.")
        }
        else {
            // TODO: add better error handling
            alert(`Failed to delete data. ${JSON.stringify(result.data)}`);
        }

    }

    render(): JSX.Element {
        return <button 
            onClick={this.apiDeleteData.bind(this)} 
            disabled={this.props.dataAccess === null}
        >
            Delete
        </button>
    }
}


export default DeleteDataAccessBtn