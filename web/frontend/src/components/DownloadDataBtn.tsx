import React from "react";
import { AxiosResponse } from "axios";

import { getApi } from "../apis/Requests";
import ResponseDataAccess from "../intefaces/ResponseDataAccess";


interface DownloadDataBtnState { }

interface DownloadDataBtnProps {
    dataAccess: ResponseDataAccess | null
}

class DownloadDataBtn extends React.Component<DownloadDataBtnProps, DownloadDataBtnState> {

    async apiDownloadData(): Promise<void> {
        if (!this.props.dataAccess) {
            return
        }
        const url = `/data-accesses/${this.props.dataAccess.id}/download`;
        const result = (await getApi(url)) as AxiosResponse;
        if (result.status === 200) {
            alert("Data is successfully received.")
        }
        else {
            // TODO: add better error handling
            alert(`Failed to download data. ${JSON.stringify(result.data)}`);
        }

    }

    render(): JSX.Element {
        return <button 
            onClick={this.apiDownloadData.bind(this)} 
            disabled={this.props.dataAccess === null}
        >
            Download
        </button>
    }
}


export default DownloadDataBtn