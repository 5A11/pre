import React from "react";
import { AxiosResponse } from "axios";
import Cookies from "js-cookie";

import { postApi } from "../apis/Requests";

interface LogoutFormState { }

interface LogoutFormProps {
    setIsLoggedIn: (value: boolean) => void;
}

class LogoutForm extends React.Component<LogoutFormProps, LogoutFormState> {
    async handleSubmit(event: React.FormEvent<HTMLFormElement>): Promise<void> {
        event.preventDefault();
        await this.apiLogout();
    }

    async apiLogout(): Promise<void> {
        const url = "/rest-auth/logout/";
        const result = (await postApi(url)) as AxiosResponse;
        if (result.status === 200) {
            Cookies.remove("csrftoken");
            this.props.setIsLoggedIn(false);
        } else {
            // TODO: add better error handling
            alert(`Failed to logout. ${JSON.stringify(result.data)}`);
        }
    }

    render(): JSX.Element {
        return (
            <div>
                <form onSubmit={this.handleSubmit.bind(this)}>
                    <button type="submit">Logout</button>
                </form>
            </div>
        );
    }
}

export default LogoutForm;
