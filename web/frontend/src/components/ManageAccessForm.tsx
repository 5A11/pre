import React from "react";
import { AxiosResponse } from "axios";

import { postApi } from "../apis/Requests";
import ResponseDataAccess from "../intefaces/ResponseDataAccess";

import style from "./ManageAccessFormForm.module.scss";


interface ManageAccessFormState {
    readers: Array<string>
    displayAdd: boolean
    displayRevoke: boolean
 }

interface ManageAccessFormProps {
    dataAccess: ResponseDataAccess | null
}

class ManageAccessForm extends React.Component<ManageAccessFormProps, ManageAccessFormState> {

    constructor(props: ManageAccessFormProps) {
        super(props);
        
        let readers: Array<string> = []
        if (this.props.dataAccess) {
            readers = this.props.dataAccess.readers
        }
        this.state = {
            readers: readers,
            displayAdd: false,
            displayRevoke: false
        };
    }

    handleAddBtnClick(): void {
        this.setState({ displayAdd: !this.state.displayAdd })
    }

    handleRevokeBtnClick(): void {
        this.setState({ displayRevoke: !this.state.displayRevoke })
    }

    async apiUpdateAccess(readers: Array<string>): Promise<void> {
        if (!this.props.dataAccess) {
            return
        }
        const url = `/data-accesses/${this.props.dataAccess?.id}/`;
        const data = {
            readers: this.state.readers
        };
        const result = (await postApi(url, data, false, true)) as AxiosResponse;
        if (result.status === 200) {
            alert(`Data access is successfully updated: ${JSON.stringify(result.data)}`)
        }
        else {
            // TODO: add better error handling
            alert(`Failed to update data access. ${JSON.stringify(result.data)}`);
        }

    }

    renderAddForm(): JSX.Element {
        // TODO: Add real form
        return <div className={style.Form}>
            <h3>Add access</h3>
        </div>
    } 

    renderRevokeForm(): JSX.Element {
        // TODO: Add real form
        return <div className={style.Form}>
            <h3>Revoke access</h3>
        </div>
    } 

    render(): JSX.Element {
        return <div className={style.Body}>
            <button 
                disabled={!this.props.dataAccess}
                onClick={this.handleAddBtnClick.bind(this)}
            >
                + Add
            </button>
            <button 
                disabled={!this.props.dataAccess}
                onClick={this.handleRevokeBtnClick.bind(this)}
            >
                - Revoke
            </button>

            { (this.props.dataAccess !== null && this.state.displayAdd) && this.renderAddForm() }
            { (this.props.dataAccess !== null && this.state.displayRevoke) && this.renderRevokeForm() }
        </div>
    }

}


export default ManageAccessForm;