import React from "react";
import { AxiosResponse } from "axios";

import { postApi } from "../apis/Requests";

import style from "./AddDataForm.module.scss";


interface AddDataFormState {
    showForm: boolean
    file: File | null
}

interface AddDataFormProps { }

class AddDataForm extends React.Component<AddDataFormProps, AddDataFormState> {

    constructor(props: AddDataFormProps) {
        super(props);

        this.state = {
            showForm: false,
            file: null
        };
    }

    handleAddBtnClick(): void {
        this.setState({ showForm: !this.state.showForm })
    }

    handleFileChange(event: React.ChangeEvent<HTMLInputElement>): void {
        const files = event.target.files
        if (files) {
            this.setState({ file: files[0] })
        }
    }

    async handleSubmit(event: React.FormEvent<HTMLFormElement>): Promise<void> {
        event.preventDefault();
        if (this.state.file !== null) {
            await this.apiAddData();
        }
    }

    async apiAddData(): Promise<void> {
        const url = "/data-accesses/create";
        const data = {
            file: this.state.file
        };
        const result = (await postApi(url, data, false, true)) as AxiosResponse;
        if (result.status === 201) {
            alert(`Data is successfully added: ${JSON.stringify(result.data)}`)
        }
        else {
            // TODO: add better error handling
            alert(`Failed to add data. ${JSON.stringify(result.data)}`);
        }

    }

    renderAddForm(): JSX.Element {
        return <form className={style.Form} onSubmit={this.handleSubmit.bind(this)}>
            <input type="file" onChange={this.handleFileChange.bind(this)} />
            <input type="submit" disabled={!this.state.file} />
        </form>
    }

    render(): JSX.Element {
        return <div className={style.Body}>
            <button onClick={this.handleAddBtnClick.bind(this)}>Add</button>
            {this.state.showForm && this.renderAddForm()}
        </div>
    }
}


export default AddDataForm