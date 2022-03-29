import React from "react";
import { AxiosResponse } from "axios";
import Cookies from "js-cookie";

import { postApi } from "../apis/Requests";

interface LogoutBtnState { }

interface LogoutBtnProps {
    setIsLoggedIn: (value: boolean) => void;
}

class LogoutBtn extends React.Component<LogoutBtnProps, LogoutBtnState> {

    async handleClick(event: React.MouseEvent<HTMLButtonElement, MouseEvent>): Promise<void> {
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
        return <button onClick={this.handleClick.bind(this)}>Logout</button>
    }
}

export default LogoutBtn;
