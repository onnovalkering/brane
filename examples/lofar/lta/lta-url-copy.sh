#!/bin/sh
#if script returns to quickly java does not see error code!!!
sleep 1
#default values
src_protocol="gsiftp"
dst_protocol="gsiftp"

usage()
{
	echo "usage: url_copy.sh -src-protocol <protocol> -src-host-port <host:port> -src-path <path> \
-dst-protocol <protocol> -dst-host-port <host:port> -dst-path <path> -proxy <path to proxy> \
-debug <true/false>"
        echo " "
        echo "  options  -src-host-port -src-path -dst-protocol and -dst-host-port are mandatory"
	echo "  default protocols are gsiftp"
	echo "  default values:"
	echo "    src-protocol gsiftp"
	echo "    dst-protocol gsiftp"
	echo "    debug false"
	echo "  if proxy is not specified, assume the default proxy location"
}

test_that_arg_exists()
{
	decho "test_that_arg_exists ( $1,$2)"
	if [ "$2" = "" ]
	then
		echo $1 option must be specified
		usage
		exit 1
	fi
}

decho()
{
	if [ "$debug" = "true" ]
	then
		echo $*
	fi
}


while [ ! -z "$1" ]
do
    decho parsing option $1 $2
    case "$1" in
        -get-protocols)
	       echo file
	       echo gsiftp
	       sleep 1
	       exit 0
	       ;;
        -src-protocol)
	       src_protocol=$2
	       decho "setting src_protocol to $2"
	       ;;
	-src-host-port)
	       src_host_port=$2
	       decho "setting src_host_port to $2"
	       ;;
        -src-path)
		src_path=$2
		decho "setting src_path to $2"
		;;
        -dst-protocol)
	        dst_protocol=$2
		decho "setting dst_protocol to $2"
		;;
	-dst-host-port)
	        dst_host_port=$2
		decho "setting dst_host_port to $2"
		;;
        -dst-path)
	        dst_path=$2
		decho "setting dst_path to $2"
		;;
	-debug)
	       debug=$2
	       decho "debug is $debug"
	      ;;
	-x509_user_proxy)
	       X509_USER_PROXY=$2
               chmod 600 $X509_USER_PROXY
               export X509_USER_PROXY
	       decho "X509_USER_PROXY is $X509_USER_PROXY"
	      ;;
	-x509_user_key)
	       X509_USER_KEY=$2
               export X509_USER_KEY
	       decho "X509_USER_KEY is $X509_USER_KEY"
	      ;;
	-x509_user_cert)
	       X509_USER_CERT=$2
               export X509_USER_CERT
	       decho "X509_USER_CERT is $X509_USER_CERT"
	      ;;
	-x509_user_certs_dir)
	       x509_user_certs_dir=$2
	       decho "x509_user_certs_dir is $x509_user_certs_dir"
	      ;;
	-buffer_size)
	       buffer_size=$2
	       decho "buffer-size is $buffer_size"
	      ;;
	-tcp_buffer_size)
	       tcp_buffer_size=$2
	       decho "tcp-buffer-size is $tcp_buffer_size"
	      ;;
        *)
	    	echo "unknown option : $1" >&2
	    	exit 1
	    	;;
    esac
    shift
    shift
done

test_that_arg_exists -src-host-port $src_host_port
test_that_arg_exists -src-path  $src_path
test_that_arg_exists -dst-host-port $dst_host_port
test_that_arg_exists  -dst-path  $dst_path

decho X509_USER_PROXY = $X509_USER_PROXY

src="$src_protocol://$src_host_port/$src_path"
case "$dst_path" in
    "/"*) dst="$dst_protocol://$dst_host_port/$dst_path" ;;
    *) dst="$dst_protocol://$dst_host_port/$PWD/$dst_path" ;;
esac


cmd=globus-url-copy
if [ "$debug" = "true" ]
then
	cmd="$cmd -dbg"
fi
if [ "$tcp_buffer_size" != "" ]
then
    cmd="$cmd -tcp-bs $tcp_buffer_size"
fi

if [ "$buffer_size" != "" ]
then
    cmd="$cmd -bs $buffer_size"
fi
cmd="$cmd -nodcau $src $dst"
decho $cmd
exec $cmd
