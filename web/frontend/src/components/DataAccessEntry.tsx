import React from "react";

import ResponseDataAccess from "../intefaces/ResponseDataAccess";

import style from "./DataAccessEntry.module.scss"

interface DataAccessEntryState { }

interface DataAccessEntryProps {
    dataAccess: ResponseDataAccess
    isOwned: boolean
    handleDataAccessCheckboxChange: (event: React.ChangeEvent<HTMLInputElement>) => void
    isCheckboxDisabled: boolean
}

class DataAccessEntry extends React.Component<DataAccessEntryProps, DataAccessEntryState> {

    render(): JSX.Element {
        return (
            <div className={style.Row}>
                <div className={style.CheckboxCol}>
                    <input 
                        type="checkbox" 
                        value={this.props.dataAccess.id} 
                        onChange={this.props.handleDataAccessCheckboxChange}
                        disabled={this.props.isCheckboxDisabled}
                    />
                </div>
                <div className={style.Col}>{this.props.dataAccess.data_id}</div>
                {this.props.isOwned && <div className={style.Col}>{this.props.dataAccess.readers.join(", ")}</div>}
            </div>
        );
    }
}


export default DataAccessEntry