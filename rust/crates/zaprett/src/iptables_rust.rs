use std::process::Command;

pub fn setup_iptables_rules() -> anyhow::Result<()> {
    Command::new("iptables")
        .arg("-t")
        .arg("mangle")
        .arg("-I")
        .arg("POSTROUTING")
        .arg("-j")
        .arg("NFQUEUE")
        .arg("--queue-num")
        .arg("200")
        .arg("--queue-bypass")
        .status()?;

    Command::new("iptables")
        .arg("-t")
        .arg("mangle")
        .arg("-I")
        .arg("PREROUTING")
        .arg("-j")
        .arg("NFQUEUE")
        .arg("--queue-num")
        .arg("200")
        .arg("--queue-bypass")
        .status()?;

    Command::new("iptables")
        .arg("-t")
        .arg("filter")
        .arg("-A")
        .arg("FORWARD")
        .arg("-j")
        .arg("NFQUEUE")
        .arg("--queue-num")
        .arg("200")
        .arg("--queue-bypass")
        .status()?;

    Ok(())
}

pub fn clear_iptables_rules() -> anyhow::Result<()> {
    Command::new("iptables")
        .arg("-t")
        .arg("mangle")
        .arg("-D")
        .arg("POSTROUTING")
        .arg("-j")
        .arg("NFQUEUE")
        .arg("--queue-num")
        .arg("200")
        .arg("--queue-bypass")
        .status()?;

    Command::new("iptables")
        .arg("-t")
        .arg("mangle")
        .arg("-D")
        .arg("PREROUTING")
        .arg("-j")
        .arg("NFQUEUE")
        .arg("--queue-num")
        .arg("200")
        .arg("--queue-bypass")
        .status()?;

    Command::new("iptables")
        .arg("-t")
        .arg("filter")
        .arg("-D")
        .arg("FORWARD")
        .arg("-j")
        .arg("NFQUEUE")
        .arg("--queue-num")
        .arg("200")
        .arg("--queue-bypass")
        .status()?;

    Ok(())
}
