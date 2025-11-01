use std::error;

pub fn setup_iptables_rules() -> Result<(), Box<dyn error::Error>> {
    let ipt = iptables::new(false)?;

    ipt.insert(
        "mangle",
        "POSTROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
        1,
    )?;

    ipt.insert(
        "mangle",
        "PREROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
        1,
    )?;

    ipt.append(
        "filter",
        "FORWARD",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )?;

    Ok(())
}

pub fn clear_iptables_rules() -> Result<(), Box<dyn error::Error>> {
    let ipt = iptables::new(false)?;

    ipt.delete(
        "mangle",
        "POSTROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )?;

    ipt.delete(
        "mangle",
        "PREROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )?;

    ipt.delete(
        "filter",
        "FORWARD",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )?;

    Ok(())
}
