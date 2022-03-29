import React from "react";

import ResponseUser from "../intefaces/ResponseUser"

import LogoutBtn from "./LogoutBtn";

import style from "./Header.module.scss"


interface HeaderState { }

interface HeaderProps {
    setIsLoggedIn: (value: boolean) => void
    responseUser: ResponseUser | null
    isLoggedIn: boolean
}

class Header extends React.Component<HeaderProps, HeaderState> {

    render(): JSX.Element {
        return <div className={style.Header}>
            <div className={style.ColLeft}>
                <h2>Proxy Re-Encryption</h2>
            </div>
            {
                this.props.responseUser
                && <div className={style.ColRight}>
                    Welcome, {this.props.responseUser.username}
                    <LogoutBtn setIsLoggedIn={this.props.setIsLoggedIn}/>
                </div>
            }
            
        </div>
    }

}

export default Header